//! The three-thread runtime.

use std::sync::mpsc;
use std::time::Duration;

use serde_json::{json, Map, Value};

use crate::error::AppResult;
use crate::features::{FeatureExtractor, FEATURE_NAMES};
use crate::ml::{Classification, Classifier};
use crate::sensor::{AccelSample, SampleSource};
use crate::tag_log;
use crate::transport::DataSink;
use crate::util::now_ms;

/// Bounded capacity for both channels. Four slots each is enough to absorb a
/// short stall without growing the heap.
const CHANNEL_CAPACITY: usize = 4;

/// Wire the sensor, classifier and sink into three scoped threads and block
/// until they all exit. On the happy path none of them ever exits, so this call
/// does not return for the lifetime of the device.
///
/// Generic over the three traits (Dependency Inversion): call sites pass
/// concrete `WindowedSampler<Mpu6050>`, `Box<dyn Classifier + Send>` and
/// `HttpSink`, but `run` itself only knows the traits.
pub fn run<Src, Clf, Sink>(
    source: Src,
    classifier: Clf,
    sink: Sink,
    device_id: &'static str,
) -> AppResult<()>
where
    Src: SampleSource,
    Clf: Classifier,
    Sink: DataSink,
{
    let (win_tx, win_rx) = mpsc::sync_channel::<Vec<AccelSample>>(CHANNEL_CAPACITY);
    let (res_tx, res_rx) = mpsc::sync_channel::<Classification>(CHANNEL_CAPACITY);

    // `thread::scope` keeps the borrowed lifetimes local: the `I2cDriver<'d>`
    // inside `source` is borrowed from `main`, and the scope guarantees the
    // sensor thread has joined before that borrow ends.
    std::thread::scope(|s| {
        // --- sensor thread: read -> window -> send --------------------------
        s.spawn(move || {
            let mut source = source;
            tag_log!("sensor", "thread started");
            loop {
                match source.next_window() {
                    Ok(window) => {
                        tag_log!("sensor", "window ready ({} samples)", window.len());
                        // send fails only if the receiver (ML thread) dropped,
                        // which means we should stop too.
                        if win_tx.send(window).is_err() {
                            tag_log!("sensor", "ML thread gone, exiting");
                            break;
                        }
                    }
                    Err(e) => {
                        tag_log!("sensor", "window error: {e}");
                        std::thread::sleep(Duration::from_millis(100));
                    }
                }
            }
        });

        // --- ML thread: window -> features -> predict -----------------------
        s.spawn(move || {
            let classifier = classifier;
            tag_log!("ml", "thread started, model = {}", classifier.name());
            while let Ok(window) = win_rx.recv() {
                let features = FeatureExtractor::extract(&window);
                let prediction = classifier.classify(&features);
                let cls =
                    Classification::new(prediction, features, window.len(), now_ms());
                tag_log!(
                    "ml",
                    "{} -> {} (conf {:.2})",
                    classifier.name(),
                    cls.activity,
                    cls.confidence
                );
                if res_tx.send(cls).is_err() {
                    tag_log!("ml", "net thread gone, exiting");
                    break;
                }
            }
        });

        // --- net thread: classification -> JSON -> POST ---------------------
        s.spawn(move || {
            let mut sink = sink;
            tag_log!("net", "thread started, device = {device_id}");
            while let Ok(cls) = res_rx.recv() {
                let payload = build_payload(device_id, &cls);
                if let Err(e) = sink.send(&payload) {
                    tag_log!("net", "send failed: {e}; backing off");
                    // Avoid hammering a down server: brief back-off.
                    std::thread::sleep(Duration::from_secs(1));
                }
            }
            tag_log!("net", "ML thread gone, exiting");
        });
    });

    // Reached only if every thread exited (e.g. fatal cascade). Park main so the
    // device does not return to the bootloader; the watchdog will eventually
    // reset it if truly wedged.
    log::error!("pipeline exited; parking main");
    loop {
        std::thread::sleep(Duration::from_secs(5));
    }
}

/// Serialise a [`Classification`] into the JSON the server expects:
///
/// ```json
/// { "device":"esp32-001", "activity":"walk", "confidence":0.6,
///   "features":{"mean_mag":1.1,"std_mag":0.3,...},
///   "samples":25, "timestamp_ms":12345 }
/// ```
fn build_payload(device: &str, cls: &Classification) -> String {
    let mut features = Map::new();
    for (name, value) in FEATURE_NAMES.iter().zip(cls.features.iter()) {
        features.insert((*name).to_string(), Value::from(*value));
    }

    json!({
        "device": device,
        "activity": cls.activity.label(),
        "confidence": cls.confidence,
        "features": Value::Object(features),
        "samples": cls.samples,
        "timestamp_ms": cls.timestamp_ms,
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sensor::Activity;

    #[test]
    fn payload_contains_activity_and_device() {
        let cls = Classification::new(
            crate::ml::Prediction {
                activity: Activity::Walk,
                confidence: 0.6,
            },
            vec![1.1, 0.3, 1.8, 0.4, 1.4],
            25,
            12345,
        );
        let s = build_payload("esp32-001", &cls);
        let v: Value = serde_json::from_str(&s).unwrap();
        assert_eq!(v["device"], "esp32-001");
        assert_eq!(v["activity"], "walk");
        assert_eq!(v["confidence"], 0.6);
        assert_eq!(v["features"]["mean_mag"], 1.1);
        assert_eq!(v["samples"], 25);
    }
}
