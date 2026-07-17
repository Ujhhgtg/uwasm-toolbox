/// Browser console logging macros for WASM.
///
/// Usage:
///   log!("step {}: doing thing", n);
///   warn!("unexpected value: {}", v);
///   error!("failed: {}", e);

use web_sys::console;
use wasm_bindgen::JsValue;

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
