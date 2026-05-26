// LbData* — game data file loading (TAB/DAT pairs).

use crate::types::*;
use std::fs;
use std::path::Path;
use std::ffi::CStr;

pub fn LbDataLoadAll(files: *const TbLoadFiles) -> TbResult {
    unsafe {
        if files.is_null() { return LB_FAIL; }
        let mut p = files;
        loop {
            if (*p).fname.is_null() { break; }
            let name = CStr::from_ptr((*p).fname).to_str().unwrap_or("");
            if name.is_empty() { break; }

            match fs::read(Path::new(name)) {
                Ok(data) => {
                    let size = data.len();
                    let buf = libc::malloc(size) as *mut u8;
                    if !buf.is_null() {
                        std::ptr::copy_nonoverlapping(data.as_ptr(), buf, size);
                        if !(*p).pp.is_null() {
                            *(*p).pp = buf as *mut libc::c_void;
                        }
                        if !(*p).fsize.is_null() {
                            *(*p).fsize = size as libc::c_long;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("LbDataLoadAll: cannot load '{}': {}", name, e);
                }
            }
            p = p.add(1);
        }
        LB_SUCCESS
    }
}

pub fn LbDataFreeAll(files: *const TbLoadFiles) {
    unsafe {
        if files.is_null() { return; }
        let mut p = files;
        loop {
            if (*p).fname.is_null() { break; }
            if !(*p).pp.is_null() && !(*(*p).pp).is_null() {
                libc::free(*(*p).pp);
                *(*p).pp = std::ptr::null_mut();
            }
            p = p.add(1);
        }
    }
}
