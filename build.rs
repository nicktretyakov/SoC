// Build script for the esp-idf-sys based project.
// Propagates the ESP-IDF SDK configuration (sdkconfig, partition table, esp-idf
// version from .cargo/config.toml) into the crate's compile-time environment.
fn main() {
    embuild::espidf::sysenv::output();
}
