//! WASM-compatible console logging.
//!
//! Provides `clog!`, `cwarn!`, and `cerror!` macros that format
//! messages with a `[uwasm]` prefix and dispatch to `web_sys::console`.
//! On WASM targets these call `console.log`/`.warn`/`.error` in the
//! browser; on native targets they print to stderr (used by tests/benchmarks).
//!
//! Usage:
//!   clog!("step {}: doing thing", n);
//!   cwarn!("unexpected value: {}", v);
//!   cerror!("failed: {}", e);

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use web_sys::console;

pub fn log(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    console::log_1(&JsValue::from_str(msg));
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("{msg}");
}

pub fn warn(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    console::warn_1(&JsValue::from_str(msg));
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("WARN {msg}");
}

pub fn error(msg: &str) {
    #[cfg(target_arch = "wasm32")]
    console::error_1(&JsValue::from_str(msg));
    #[cfg(not(target_arch = "wasm32"))]
    eprintln!("ERROR {msg}");
}

#[macro_export]
macro_rules! clog {
    ($($arg:tt)*) => { $crate::log::log(&format!("[uwasm] {}", format_args!($($arg)*))) };
}

#[macro_export]
macro_rules! cwarn {
    ($($arg:tt)*) => { $crate::log::warn(&format!("[uwasm] {}", format_args!($($arg)*))) };
}

#[macro_export]
macro_rules! cerror {
    ($($arg:tt)*) => { $crate::log::error(&format!("[uwasm] {}", format_args!($($arg)*))) };
}
