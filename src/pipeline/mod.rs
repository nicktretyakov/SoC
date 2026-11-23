//! Pipeline: three scoped threads wired by bounded `mpsc` channels.
//!
//! ```text
//!   sensor thread ──Vec<AccelSample>──▶ ml thread ──Classification──▶ net thread
//! ```
//!
//! Why `std::thread::scope` (not bare `thread::spawn`)? The sensor owns an
//! `I2cDriver<'d>` whose lifetime is tied to the `I2cConfig` borrowed in `main`.
//! Scoped threads can borrow non-`'static` data, so the driver does not need to
//! be leaked to `'static` — the scope guarantees all threads have joined before
//! the borrowed config is dropped.
//!
//! Why bounded `sync_channel(4)`? Back-pressure: if the ML thread is slow, the
//! sensor thread blocks instead of buffering unbounded windows into heap. On an
//! ESP32 with ~300 KB of heap this matters.

pub mod threads;

pub use threads::run;
