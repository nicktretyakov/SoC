//! Small host-agnostic helpers.

/// Monotonic milliseconds since boot, sourced from the ESP-IDF high-resolution
/// timer. Used as a coarse timestamp in HTTP payloads.
pub fn now_ms() -> u32 {
    // `esp_timer_get_time` returns i64 microseconds; u32 wraps after ~49.7 days
    // which is fine for relative timestamps in this demo.
    let us: i64 = unsafe { esp_idf_sys::esp_timer_get_time() };
    (us / 1000) as u32
}

/// `Duration` between two `now_ms()` readings, robust against the u32 wrap.
pub fn elapsed_ms(start_ms: u32, end_ms: u32) -> u32 {
    end_ms.wrapping_sub(start_ms)
}
