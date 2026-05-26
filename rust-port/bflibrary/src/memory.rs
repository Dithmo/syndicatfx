// LbMemory* — thin wrappers around the system allocator.

use crate::types::*;
use std::alloc::{alloc, dealloc, Layout};

pub fn LbMemoryAlloc(size: TbMemSize) -> *mut u8 {
    if size == 0 {
        return std::ptr::null_mut();
    }
    unsafe {
        let layout = Layout::from_size_align(size, 8).unwrap();
        let ptr = alloc(layout);
        if !ptr.is_null() {
            std::ptr::write_bytes(ptr, 0, size);
        }
        ptr
    }
}

pub fn LbMemoryFree(ptr: *mut u8) {
    if ptr.is_null() {
        return;
    }
    // Cannot safely dealloc without knowing the size; use libc free instead.
    unsafe { libc::free(ptr as *mut libc::c_void); }
}

pub fn LbMemoryReset() {
    // Nothing to do — allocations are owned by callers.
}

pub fn LbMemoryCopy(dst: *mut u8, src: *const u8, size: usize) {
    if size == 0 || dst.is_null() || src.is_null() {
        return;
    }
    unsafe { std::ptr::copy_nonoverlapping(src, dst, size); }
}

pub fn LbMemorySet(dst: *mut u8, val: u8, size: usize) {
    if size == 0 || dst.is_null() {
        return;
    }
    unsafe { std::ptr::write_bytes(dst, val, size); }
}

pub fn LbMemoryMove(dst: *mut u8, src: *const u8, size: usize) {
    if size == 0 || dst.is_null() || src.is_null() {
        return;
    }
    unsafe { std::ptr::copy(src, dst, size); }
}
