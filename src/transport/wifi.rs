//! Wi-Fi station connect, using `esp-idf-svc` 0.48.
//!
//! Returns a `BlockingWifi` handle that the caller must keep alive for as long
//! as HTTP requests are made: dropping it tears the interface down.

use std::time::Duration;

use esp_idf_svc::event::EspSystemEventLoop;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::{BlockingWifi, Configuration, EspWifi, WifiConfiguration};

use crate::error::{AppError, AppResult};
use crate::tag_log;

/// Connect to the given WPA-PSK network as a station and block until DHCP has
/// assigned an IP. The returned handle owns the radio and the netif; keep it
/// alive in `main` for the lifetime of the program.
///
/// `nvs` is the partition returned by `EspDefaultNvsPartition::take()`; pass
/// `None` only for very constrained builds without NVS.
pub fn connect(
    sys_loop: &EspSystemEventLoop,
    nvs: Option<EspDefaultNvsPartition>,
    ssid: &str,
    password: &str,
) -> AppResult<BlockingWifi<EspWifi<'static>>> {
    let mut wifi =
        BlockingWifi::wrap(EspWifi::new(sys_loop, nvs).map_err(AppError::wifi)?);

    wifi.set_configuration(&Configuration::Client(WifiConfiguration {
        ssid: ssid.into(),
        password: password.into(),
        ..Default::default()
    }))
    .map_err(AppError::wifi)?;

    wifi.start().map_err(AppError::wifi)?;
    tag_log!("wifi", "started, connecting to SSID `{}`", ssid);

    // `connect` + `wait_netif_up` block until the link is up. There is no
    // per-call timeout in the blocking API; a production build would poll with
    // a deadline and fall back to `LoggingSink` if the AP is unreachable.
    wifi.connect().map_err(AppError::wifi)?;
    wifi.wait_netif_up().map_err(AppError::wifi)?;

    // Give LWIP a moment to settle before HTTP bursts.
    std::thread::sleep(Duration::from_millis(250));

    let ip = wifi
        .sta_netif()
        .ok_or_else(|| AppError::Wifi("sta_netif missing".into()))?
        .ip_info()
        .map_err(AppError::wifi)?
        .ip;
    tag_log!("wifi", "connected, IP = {}", ip);
    Ok(wifi)
}
