// display.rs — mirrors display.c: VGA buffer management and screen blitting.

use bflibrary::memory::{LbMemoryAlloc, LbMemoryFree, LbMemoryCopy, LbMemorySet};
use bflibrary::screen::{LbScreenLock, LbScreenUnlock, LbScreenIsLocked,
                         LbScreenSwap, LbScreenSetMinScreenSurfaceDimension,
                         LbScreenSetup, LbScreenGetModeInfo, LB_DISPLAY};
use bflibrary::mouse::LbMouseChangeMoveRatio;
use bflibrary::types::*;
use crate::globals::*;

// ---- AppResourceMapping ---------------------------------------------------

pub fn app_resource_mapping(index: i16) -> *const libc::c_char {
    match index {
        1 => b"syndicatfx_icon64.png\0".as_ptr() as *const libc::c_char,
        _ => std::ptr::null(),
    }
}

// ---- Screen mode setup ----------------------------------------------------

pub fn app_screen_setup(mode: u16) -> TbResult {
    unsafe {
        let mut m = mode;
        if m == 19 { m = LB_SCREEN_MODE_320_200_8; }
        else if m == 18 { m = LB_SCREEN_MODE_640_480_8; }

        LB_DISPLAY.w_screen = VGA_BUFFER.add(640 * 480);

        let mdinfo = LbScreenGetModeInfo(m);
        let (w, h) = if mdinfo.is_null() { (640, 480) } else {
            ((*mdinfo).width, (*mdinfo).height)
        };

        let ret = LbScreenSetup(m, w, h, GRAPHICS_PALETTE);
        LbMouseChangeMoveRatio(1 << 8, 1 << 8);
        bflibrary::mouse::LbMouseSetWindow(
            0, 0, LB_DISPLAY.graphics_screen_width, LB_DISPLAY.graphics_screen_height);
        ret
    }
}

// ---- Full-screen / stretch ------------------------------------------------

pub fn display_set_full_screen(full_screen: bool) {
    // SDL full-screen is handled at window creation in LbScreenSetup.
    // Here we toggle the windowed flag in mode info records.
    for i in 1..LB_MAX_SCREEN_MODES_COUNT {
        let mdinfo = LbScreenGetModeInfo(i as u16);
        if mdinfo.is_null() { break; }
        unsafe {
            if (*mdinfo).width == 0 { break; }
            if full_screen {
                (*mdinfo).video_mode &= !LB_VF_WINDOWED;
            } else {
                (*mdinfo).video_mode |= LB_VF_WINDOWED;
            }
        }
    }
}

pub fn display_set_lowres_stretch(stretch: bool) {
    if stretch {
        LbScreenSetMinScreenSurfaceDimension(400);
    } else {
        LbScreenSetMinScreenSurfaceDimension(1);
    }
}

// ---- VGA buffer -----------------------------------------------------------

pub fn display_create_vga_buffer() {
    unsafe {
        VGA_BUFFER = LbMemoryAlloc(640 * 480 * 2 + 16);
    }
}

pub fn display_free_vga_buffer() {
    unsafe {
        LbMemoryFree(VGA_BUFFER);
        VGA_BUFFER = std::ptr::null_mut();
    }
}

pub fn display_lock() {
    LbScreenLock();
}

pub fn display_unlock() {
    LbScreenUnlock();
}

// ---- Planar → 8-bit conversion (vres16 mode) ------------------------------
// The game stores frames in 4-plane EGA format: 4 consecutive 400×80-byte
// planes.  This function unpacks them into a linear 640×480 8-bit buffer.

pub fn update_vscreen_whole_vres16() {
    unsafe {
        if WSCREEN.is_null() || VSCREEN.is_null() { return; }
        LbMemorySet(VSCREEN, 0, 640 * 480);

        // Plane 0 → bit 0
        let mut i = WSCREEN.add(0 * 400 * 80);
        let mut o = VSCREEN;
        for _ in 0..400 {
            for _ in 0..80 {
                let b = *i;
                *o.add(7) |= (b >> 0) & 0x01;
                *o.add(6) |= (b >> 1) & 0x01;
                *o.add(5) |= (b >> 2) & 0x01;
                *o.add(4) |= (b >> 3) & 0x01;
                *o.add(3) |= (b >> 4) & 0x01;
                *o.add(2) |= (b >> 5) & 0x01;
                *o.add(1) |= (b >> 6) & 0x01;
                *o.add(0) |= (b >> 7) & 0x01;
                o = o.add(8);
                i = i.add(1);
            }
        }
        // Plane 1 → bit 1
        i = WSCREEN.add(1 * 400 * 80);
        o = VSCREEN;
        for _ in 0..400 {
            for _ in 0..80 {
                let b = *i;
                *o.add(7) |= (b << 1) & 0x02;
                *o.add(6) |= (b >> 0) & 0x02;
                *o.add(5) |= (b >> 1) & 0x02;
                *o.add(4) |= (b >> 2) & 0x02;
                *o.add(3) |= (b >> 3) & 0x02;
                *o.add(2) |= (b >> 4) & 0x02;
                *o.add(1) |= (b >> 5) & 0x02;
                *o.add(0) |= (b >> 6) & 0x02;
                o = o.add(8);
                i = i.add(1);
            }
        }
        // Plane 2 → bit 2
        i = WSCREEN.add(2 * 400 * 80);
        o = VSCREEN;
        for _ in 0..400 {
            for _ in 0..80 {
                let b = *i;
                *o.add(7) |= (b << 2) & 0x04;
                *o.add(6) |= (b << 1) & 0x04;
                *o.add(5) |= (b >> 0) & 0x04;
                *o.add(4) |= (b >> 1) & 0x04;
                *o.add(3) |= (b >> 2) & 0x04;
                *o.add(2) |= (b >> 3) & 0x04;
                *o.add(1) |= (b >> 4) & 0x04;
                *o.add(0) |= (b >> 5) & 0x04;
                o = o.add(8);
                i = i.add(1);
            }
        }
        // Plane 3 → bit 3
        i = WSCREEN.add(3 * 400 * 80);
        o = VSCREEN;
        for _ in 0..400 {
            for _ in 0..80 {
                let b = *i;
                *o.add(7) |= (b << 3) & 0x08;
                *o.add(6) |= (b << 2) & 0x08;
                *o.add(5) |= (b << 1) & 0x08;
                *o.add(4) |= (b >> 0) & 0x08;
                *o.add(3) |= (b >> 1) & 0x08;
                *o.add(2) |= (b >> 2) & 0x08;
                *o.add(1) |= (b >> 3) & 0x08;
                *o.add(0) |= (b >> 4) & 0x08;
                o = o.add(8);
                i = i.add(1);
            }
        }
    }
}

// ---- Screen blitting ------------------------------------------------------

pub fn blit_wscreen_vres16(buf: *const u8) {
    unsafe {
        LbMemoryCopy(LB_DISPLAY.w_screen, buf, 640 * 480);
    }
}

pub fn blit_wscreen_mcga(buf: *const u8) {
    unsafe {
        LbMemoryCopy(LB_DISPLAY.w_screen, buf, 320 * 200);
    }
}

pub fn swap_wscreen() {
    unsafe {
        let was_locked = LbScreenIsLocked() != 0;
        if !was_locked { LbScreenLock(); }
        if (DRAW_FLAGS & DRW_F_SCREEN_VRES16) != 0 {
            blit_wscreen_vres16(VGA_BUFFER);
        } else {
            blit_wscreen_mcga(WSCREEN);
        }
        LbScreenUnlock();
        LbScreenSwap();
        if was_locked {
            while LbScreenLock() != LB_SUCCESS {}
        }
    }
}
