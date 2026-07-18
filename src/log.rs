//! WASM-compatible console logging.
//!
//! Provides `clog!`, `cwarn!`, and `cerror!` macros that format
//! messages with a `[uwasm]` prefix and dispatch to `web_sys::console`.
//! On WASM targets these call `console.log`/`.warn`/`.error` in the
//! browser; they are no-ops in terms of panicking but still allocate.
//!
//! Usage:
//!   clog!("step {}: doing thing", n);
//!   cwarn!("unexpected value: {}", v);
//!   cerror!("failed: {}", e);

use wasm_bindgen::JsValue;
use web_sys::console;

pub fn log(msg: &str) {
    console::log_1(&JsValue::from_str(msg));
}

pub fn warn(msg: &str) {
    console::warn_1(&JsValue::from_str(msg));
}

pub fn error(msg: &str) {
    console::error_1(&JsValue::from_str(msg));
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
