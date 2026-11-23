//! Compile-time configuration.
//!
//! Everything the user is expected to edit lives here: Wi-Fi credentials, the
//! HTTP endpoint, the device id, the I2C address and the sampling window. These
//! are constants rather than runtime config because on an embedded target a
//! `const` costs nothing in RAM and is impossible to forget to set.

use crate::activity_label;

// --- Activity labels (declared via macro) -------------------------------------
activity_label!(LABEL_REST, "rest");
activity_label!(LABEL_WALK, "walk");
activity_label!(LABEL_RUN, "run");

/// Which ML model to instantiate at boot. Either `"knn"` (hand-rolled k-NN) or
/// `"decision_tree"` (smartcore). Anything else falls back to the decision tree.
pub const ML_MODEL: &str = "knn";

// --- Wi-Fi / network ----------------------------------------------------------
/// Replace with your 2.4 GHz SSID. ESP32 classic Wi-Fi only.
pub const WIFI_SSID: &str = "your_ssid";
/// Replace with your Wi-Fi password.
pub const WIFI_PASS: &str = "your_password";
/// Replace with the URL that accepts the JSON POST, e.g. `http://192.168.1.10:8080/data`.
pub const SERVER_URL: &str = "http://your_server_url/data";
/// Identifier sent in every payload so the server can tell devices apart.
pub const DEVICE_ID: &str = "esp32-001";

// --- I2C / MPU6050 ------------------------------------------------------------
/// MPU6050 7-bit I2C address when AD0 is tied to GND. Use 0x69 if AD0 is high.
pub const I2C_MASTER_ADDR: u8 = 0x68;
/// SDA pin. Matches the wiring in the README (GPIO21).
pub const SDA_PIN: u8 = 21;
/// SCL pin. Matches the wiring in the README (GPIO22).
pub const SCL_PIN: u8 = 22;
/// I2C clock. 400 kHz is within MPU6050 specs and stable on short wires.
pub const I2C_BAUDRATE_HZ: u32 = 400_000;

// --- Sampling / windowing -----------------------------------------------------
/// Target accelerometer sample rate in Hz.
pub const SAMPLE_HZ: u32 = 50;
/// Number of samples aggregated into one feature window. At 50 Hz this is 0.5 s,
/// short enough to feel responsive yet long enough to estimate variance.
pub const WINDOW_SAMPLES: usize = 25;

// --- ML hyper-parameters ------------------------------------------------------
/// `k` for the k-NN classifier. Odd to avoid ties with 3 classes.
pub const KNN_K: usize = 5;

/// Decision-tree training parameters (forwarded to smartcore).
pub const DT_MIN_SAMPLES_LEAF: usize = 1;
pub const DT_MIN_SAMPLES_SPLIT: usize = 2;
pub const DT_MAX_DEPTH: usize = 8;
