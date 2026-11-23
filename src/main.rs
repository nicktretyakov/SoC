//! Entry point.
//!
//! Wires the layers together:
//! 1. Take the ESP32 peripherals, build an I2C driver on GPIO21/22, and wrap it
//!    in `Mpu6050` + `WindowedSampler`.
//! 2. Bring up Wi-Fi (station mode) and block until an IP is assigned.
//! 3. Train (or memorise) the chosen ML model behind a `Box<dyn Classifier>`.
//! 4. Build an `HttpSink` (or a `LoggingSink` if `SERVER_URL` is still the
//!    placeholder) behind a `Box<dyn DataSink>`.
//! 5. Hand everything to `pipeline::run`, which spawns the three worker
//!    threads and blocks forever.

use anyhow::Context;

use esp_idf_hal::i2c::{I2cConfig, I2cDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::prelude::*;
use esp_idf_hal::units::Hertz;

use esp_idf_svc::event::EspSystemEventLoop;
use esp_idf_svc::log::EspLogger;
use esp_idf_svc::nvs::EspDefaultNvsPartition;

use esp_idf_sys as _; // pulls in the patches / link

use esp32_ml_activity::config;
use esp32_ml_activity::ml::dataset;
use esp32_ml_activity::sensor::{Mpu6050, WindowedSampler};
use esp32_ml_activity::transport::wifi;
use esp32_ml_activity::{
    Classifier, DataSink, DecisionTreeClassifierModel, HttpSink, KnnClassifier, LoggingSink,
};
use esp32_ml_activity::pipeline;

fn main() -> anyhow::Result<()> {
    // Bind ESP-IDF patches and route `log` to serial.
    esp_idf_sys::link_patches();
    let _ = EspLogger::initialize_default();

    log::info!("booting esp32-ml-activity v{}", env!("CARGO_PKG_VERSION"));

    // --- 1. Hardware: I2C + MPU6050 -----------------------------------------
    let peripherals = Peripherals::take().context("peripherals already taken")?;
    let pins = peripherals.pins;
    let i2c = peripherals.i2c0;

    let i2c_config = I2cConfig::new().baudrate(Hertz::from(config::I2C_BAUDRATE_HZ));
    let driver = I2cDriver::new(i2c, pins.gpio21, pins.gpio22, &i2c_config)
        .context("I2cDriver::new")?;
    // `driver` borrows `i2c_config`; both live in `main`, so the borrow
    // outlives the scoped threads started below.

    let sensor = Mpu6050::new(driver, config::I2C_MASTER_ADDR).context("MPU6050 init")?;
    let source = WindowedSampler::new(sensor, config::WINDOW_SAMPLES, config::SAMPLE_HZ);

    // --- 2. Network: Wi-Fi ---------------------------------------------------
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    // Keep the handle alive for the whole program; dropping it would tear the
    // Wi-Fi interface down.
    let _wifi = wifi::connect(
        &sys_loop,
        Some(nvs),
        config::WIFI_SSID,
        config::WIFI_PASS,
    )
    .context("Wi-Fi connect")?;

    // --- 3. ML model ---------------------------------------------------------
    let dataset = dataset::build_dataset();
    log::info!("dataset: {} samples", dataset.len());

    let classifier: Box<dyn Classifier + Send> = match config::ML_MODEL {
        "knn" => {
            log::info!("using k-NN (k={})", config::KNN_K);
            Box::new(KnnClassifier::new(&dataset, config::KNN_K)?)
        }
        "decision_tree" => {
            log::info!("using smartcore decision tree");
            Box::new(DecisionTreeClassifierModel::train(&dataset)?)
        }
        other => {
            log::warn!("unknown ML_MODEL `{other}`, falling back to decision tree");
            Box::new(DecisionTreeClassifierModel::train(&dataset)?)
        }
    };

    // --- 4. Sink: HTTP if configured, else logging ---------------------------
    let sink: Box<dyn DataSink + Send> = if config::SERVER_URL.contains("your_server_url") {
        log::warn!("SERVER_URL not set; using LoggingSink (no network POST)");
        Box::new(LoggingSink::new())
    } else {
        Box::new(HttpSink::new(config::SERVER_URL)?)
    };

    // --- 5. Run the pipeline (blocks forever) --------------------------------
    pipeline::run(source, classifier, sink, config::DEVICE_ID)?;
    Ok(())
}
