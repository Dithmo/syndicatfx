// dos.rs — mirrors dos.c: DOS-compatibility shim for file I/O, time, paths.

use std::ffi::{CStr, CString};
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::collections::HashMap;
use std::os::unix::fs::OpenOptionsExt;
use libc;

// DOS open flags
pub const DOS_O_RDONLY:    i32 = 0x0000;
pub const DOS_O_WRONLY:    i32 = 0x0001;
pub const DOS_O_RDWR:      i32 = 0x0002;
pub const DOS_O_APPEND:    i32 = 0x0010;
pub const DOS_O_CREAT:     i32 = 0x0020;
pub const DOS_O_TRUNC:     i32 = 0x0040;
pub const DOS_O_NOINHERIT: i32 = 0x0080;
pub const DOS_O_TEXT:      i32 = 0x0100;
pub const DOS_O_BINARY:    i32 = 0x0200;
pub const DOS_O_EXCL:      i32 = 0x0400;

pub const DOS_CLOCKS_PER_SEC: u32 = 100;

#[repr(C)]
pub struct DosTime {
    pub hour:    u8,
    pub minute:  u8,
    pub second:  u8,
    pub hsecond: u8,
}

#[repr(C)]
pub struct DosDate {
    pub day:        u8,
    pub month:      u8,
    pub year:       u16,
    pub dayofweek:  u8,
}

// ---- Path conversion -------------------------------------------------------

pub fn dos_path_to_native(path: &str, buf: &mut [u8]) -> bool {
    let native = path.replace('\\', "/");
    // Case-insensitive lookup on Unix
    let resolved = lookup_path_case_insensitive(&native);
    let bytes = resolved.as_bytes();
    let len = bytes.len().min(buf.len() - 1);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf[len] = 0;
    true
}

fn lookup_path_case_insensitive(path: &str) -> String {
    // Walk each segment, doing case-insensitive match on the filesystem.
    let mut result = String::new();
    let parts: Vec<&str> = path.split('/').collect();
    for (i, seg) in parts.iter().enumerate() {
        if seg.is_empty() { continue; }
        let dir = if result.is_empty() { ".".to_string() } else { result.clone() };
        if let Ok(entries) = fs::read_dir(&dir) {
            let mut matched = false;
            for entry in entries.flatten() {
                let name = entry.file_name();
                let name_str = name.to_string_lossy();
                if name_str.to_lowercase() == seg.to_lowercase() {
                    if result.is_empty() { result = name_str.to_string(); }
                    else { result = format!("{}/{}", result, name_str); }
                    matched = true;
                    break;
                }
            }
            if !matched {
                if result.is_empty() { result = seg.to_lowercase(); }
                else { result = format!("{}/{}", result, seg.to_lowercase()); }
            }
        } else {
            if result.is_empty() { result = seg.to_lowercase(); }
            else { result = format!("{}/{}", result, seg.to_lowercase()); }
        }
    }
    if result.is_empty() { path.to_string() } else { result }
}

// ---- File I/O (forwarded to libc open/read/write/lseek/close) -------------

pub fn dos_open(path: &str, flags: i32) -> i32 {
    let native = {
        let mut buf = vec![0u8; 4096];
        if !dos_path_to_native(path, &mut buf) { return -1; }
        String::from_utf8_lossy(&buf[..buf.iter().position(|&b| b==0).unwrap_or(0)]).to_string()
    };
    let c_path = match CString::new(native.clone()) {
        Ok(s) => s,
        Err(_) => return -1,
    };
    let native_flags = dos_flags_to_native(flags);
    unsafe { libc::open(c_path.as_ptr(), native_flags, 0o666) }
}

fn dos_flags_to_native(flags: i32) -> i32 {
    let mut out = 0i32;
    let access = flags & 0x03;
    out |= match access {
        0 => libc::O_RDONLY,
        1 => libc::O_WRONLY,
        2 => libc::O_RDWR,
        _ => libc::O_RDONLY,
    };
    if (flags & DOS_O_APPEND) != 0 { out |= libc::O_APPEND; }
    if (flags & DOS_O_CREAT)  != 0 { out |= libc::O_CREAT; }
    if (flags & DOS_O_TRUNC)  != 0 { out |= libc::O_TRUNC; }
    if (flags & DOS_O_EXCL)   != 0 { out |= libc::O_EXCL; }
    out
}

pub fn dos_creat(path: &str, _share: i32) -> i32 {
    dos_open(path, DOS_O_WRONLY | DOS_O_CREAT | DOS_O_TRUNC)
}

pub fn dos_lseek(fd: i32, off: i64, whence: i32) -> i64 {
    unsafe { libc::lseek(fd, off as libc::off_t, whence) as i64 }
}

pub fn dos_tell(fd: i32) -> i64 {
    dos_lseek(fd, 0, libc::SEEK_CUR)
}

pub fn dos_filelength(fd: i32) -> i64 {
    let cur = dos_tell(fd);
    let end = dos_lseek(fd, 0, libc::SEEK_END);
    dos_lseek(fd, cur, libc::SEEK_SET);
    end
}

pub fn dos_close(fd: i32) {
    unsafe { libc::close(fd); }
}

// ---- Low-level file ops (called from syndre wrapper functions) ------------

pub fn dos_low_level_open(path: &str, _ds: u16, mode: u16) -> i32 {
    let flags = match mode & 3 {
        1 => DOS_O_WRONLY,
        2 => DOS_O_RDWR,
        _ => DOS_O_RDONLY,
    };
    // Apply game_transform_path equivalent
    let tpath = crate::game::game_transform_path(path);
    dos_open(&tpath, flags | DOS_O_BINARY)
}

pub fn dos_low_level_read(fd: i32, buf: *mut u8, _ds: u16, size: usize,
                           bytes_read: *mut usize) -> usize {
    let n = unsafe { libc::read(fd, buf as *mut libc::c_void, size) };
    let n = if n < 0 { 0 } else { n as usize };
    if !bytes_read.is_null() { unsafe { *bytes_read = n; } }
    n
}

pub fn dos_low_level_seek(fd: i32, pos: u32) -> u32 {
    let r = unsafe { libc::lseek(fd, pos as libc::off_t, libc::SEEK_SET) };
    r.max(0) as u32
}

pub fn dos_low_level_seek_relative(fd: i32, pos: i32) -> u32 {
    let r = unsafe { libc::lseek(fd, pos as libc::off_t, libc::SEEK_CUR) };
    r.max(0) as u32
}

pub fn dos_low_level_close(fd: i32) {
    dos_close(fd);
}

// ---- Time -----------------------------------------------------------------

pub fn dos_gettime(t: &mut DosTime) {
    use std::time::SystemTime;
    let d = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs() % 86400;
    t.hour    = (secs / 3600) as u8;
    t.minute  = ((secs % 3600) / 60) as u8;
    t.second  = (secs % 60) as u8;
    t.hsecond = (d.subsec_millis() / 10) as u8;
}

pub fn dos_clock() -> u32 {
    let ms = unsafe { sdl2::sys::SDL_GetTicks() as u64 };
    (ms * DOS_CLOCKS_PER_SEC as u64 / 1000) as u32
}

// ---- DOS interrupt stubs (abort if called) --------------------------------

pub fn dos_int386(_num: i32) -> ! {
    panic!("dos_int386 called — unimplemented DOS interrupt");
}

pub fn dos_getvect(_num: i32) -> ! {
    panic!("dos_getvect called — unimplemented");
}

pub fn dos_setvect(_num: i32) -> ! {
    panic!("dos_setvect called — unimplemented");
}
