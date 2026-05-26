// LbLog* / LbSyncLog — logging stubs.

use crate::types::*;

pub static mut LOG_FILE: Option<std::fs::File> = None;

pub fn LbErrorLogSetup(_path: *const libc::c_char, _fname: *const libc::c_char,
                       _flags: u32) -> TbResult {
    LB_SUCCESS
}

pub fn LbErrorLogReset() {
    unsafe { LOG_FILE = None; }
}

// In Rust we can't do printf-style varargs easily; the callers use macros.
// The macros translate to calls here.
pub fn lb_sync_log_str(msg: &str) {
    eprint!("[SYNC] {}", msg);
}

pub fn lb_err_log_str(msg: &str) {
    eprint!("[ERR] {}", msg);
}

#[macro_export]
macro_rules! LOGSYNC {
    ($($arg:tt)*) => { eprintln!("[SYNC] {}", format!($($arg)*)) };
}

#[macro_export]
macro_rules! LOGERR {
    ($($arg:tt)*) => { eprintln!("[ERR] {}", format!($($arg)*)) };
}

#[macro_export]
macro_rules! LOGNO {
    ($($arg:tt)*) => { };
}
