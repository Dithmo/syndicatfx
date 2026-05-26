// LbScreen* — SDL2-backed screen management replacing bflibrary's video layer.

use crate::types::*;
use sdl2::pixels::PixelFormatEnum;
use sdl2::render::Canvas;
use sdl2::video::Window;
use std::ptr;

// ---- Global SDL state -------------------------------------------------------

pub static mut LB_DISPLAY: TbDisplayStruct = TbDisplayStruct::zero();

static mut SCREEN_MODES: [TbScreenModeInfo; LB_MAX_SCREEN_MODES_COUNT] = {
    let mut arr = [TbScreenModeInfo::zero(); LB_MAX_SCREEN_MODES_COUNT];
    // Mode 5 → 320×200×8
    arr[5] = TbScreenModeInfo { width: 320, height: 200, bits_per_pixel: 8,
                                video_mode: 0, name: *b"320x200x8\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0" };
    // Mode 13 → 640×480×8
    arr[13] = TbScreenModeInfo { width: 640, height: 480, bits_per_pixel: 8,
                                 video_mode: 0, name: *b"640x480x8\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0" };
    arr
};

pub static mut LB_SCREEN_SURFACE: *mut sdl2::sys::SDL_Surface = ptr::null_mut();
pub static mut SDL_CONTEXT: Option<sdl2::Sdl> = None;
pub static mut SDL_CANVAS:  Option<Canvas<Window>> = None;
pub static mut SCREEN_LOCKED: bool = false;
pub static mut SCREEN_BACK_BUF: Vec<u8> = Vec::new();
pub static mut TITLE: [u8; 64] = [0u8; 64];
pub static mut MIN_SURFACE_DIM: u32 = 1;

// ---- Public API ------------------------------------------------------------

pub fn LbBaseInitialise() -> TbResult {
    unsafe {
        let sdl = match sdl2::init() {
            Ok(s) => s,
            Err(_) => return LB_FAIL,
        };
        SDL_CONTEXT = Some(sdl);
        LB_SUCCESS
    }
}

pub fn LbBaseReset() -> TbResult {
    unsafe {
        SDL_CANVAS = None;
        SDL_CONTEXT = None;
        LB_SUCCESS
    }
}

pub fn LbScreenGetModeInfo(mode: u16) -> *mut TbScreenModeInfo {
    unsafe {
        let idx = mode as usize;
        if idx < LB_MAX_SCREEN_MODES_COUNT {
            &mut SCREEN_MODES[idx] as *mut TbScreenModeInfo
        } else {
            ptr::null_mut()
        }
    }
}

pub fn LbScreenSetup(mode: u16, width: u16, height: u16, palette: *mut u8) -> TbResult {
    unsafe {
        let sdl = match SDL_CONTEXT.as_ref() {
            Some(s) => s,
            None => return LB_FAIL,
        };
        let video = match sdl.video() {
            Ok(v) => v,
            Err(_) => return LB_FAIL,
        };

        let title = std::str::from_utf8(&TITLE)
            .unwrap_or("SyndicatFX")
            .trim_end_matches('\0');

        let (win_w, win_h) = if width < MIN_SURFACE_DIM as u16 {
            (MIN_SURFACE_DIM, height as u32)
        } else {
            (width as u32, height as u32)
        };

        let window = match video
            .window(title, win_w.max(width as u32), win_h.max(height as u32))
            .position_centered()
            .resizable()
            .build()
        {
            Ok(w) => w,
            Err(_) => return LB_FAIL,
        };

        let canvas = match window.into_canvas().build() {
            Ok(c) => c,
            Err(_) => return LB_FAIL,
        };

        LB_DISPLAY.graphics_screen_width  = width as i32;
        LB_DISPLAY.graphics_screen_height = height as i32;
        LB_DISPLAY.screen_mode = mode;
        LB_DISPLAY.palette = palette;

        let buf_size = (width as usize) * (height as usize);
        SCREEN_BACK_BUF = vec![0u8; buf_size];
        LB_DISPLAY.w_screen = SCREEN_BACK_BUF.as_mut_ptr();

        SDL_CANVAS = Some(canvas);
        LB_SUCCESS
    }
}

pub fn LbScreenReset() -> TbResult {
    unsafe {
        SCREEN_LOCKED = false;
        SDL_CANVAS = None;
        LB_SUCCESS
    }
}

pub fn LbScreenLock() -> TbResult {
    unsafe {
        if SCREEN_LOCKED {
            return LB_SUCCESS;
        }
        SCREEN_LOCKED = true;
        LB_SUCCESS
    }
}

pub fn LbScreenUnlock() -> TbResult {
    unsafe {
        SCREEN_LOCKED = false;
        LB_SUCCESS
    }
}

pub fn LbScreenIsLocked() -> TbBool {
    unsafe { SCREEN_LOCKED as TbBool }
}

pub fn LbScreenSwap() -> TbResult {
    unsafe {
        let canvas = match SDL_CANVAS.as_mut() {
            Some(c) => c,
            None => return LB_FAIL,
        };

        let w = LB_DISPLAY.graphics_screen_width as u32;
        let h = LB_DISPLAY.graphics_screen_height as u32;
        let palette = LB_DISPLAY.palette;

        if !palette.is_null() {
            // Build an RGB24 frame by expanding the 8-bit indexed pixels via palette.
            let src  = std::slice::from_raw_parts(LB_DISPLAY.w_screen, (w * h) as usize);
            let pal  = std::slice::from_raw_parts(palette, 256 * 3);
            let mut rgb = vec![0u8; (w * h * 3) as usize];
            for (i, &idx) in src.iter().enumerate() {
                let base = (idx as usize) * 3;
                // Bullfrog palettes are 6-bit (0-63); scale to 8-bit
                rgb[i * 3]     = pal[base]     * 4;
                rgb[i * 3 + 1] = pal[base + 1] * 4;
                rgb[i * 3 + 2] = pal[base + 2] * 4;
            }
            let surface = sdl2::surface::Surface::from_data(
                &mut rgb, w, h, w * 3, PixelFormatEnum::RGB24);
            if let Ok(surf) = surface {
                if let Ok(tex) = canvas.texture_creator()
                    .create_texture_from_surface(&surf)
                {
                    let _ = canvas.copy(&tex, None, None);
                }
            }
        }
        canvas.present();
        LB_SUCCESS
    }
}

pub fn LbScreenWaitVbi() {
    // On modern hardware there is no true VBI to wait for.
    // SDL's frame pacing in LbScreenSwap handles timing.
}

pub fn LbScreenSetMinScreenSurfaceDimension(dim: u32) {
    unsafe { MIN_SURFACE_DIM = dim; }
}

pub fn LbSetTitle(title: &str) {
    unsafe {
        let bytes = title.as_bytes();
        let len = bytes.len().min(TITLE.len() - 1);
        TITLE[..len].copy_from_slice(&bytes[..len]);
        TITLE[len] = 0;
        if let Some(canvas) = SDL_CANVAS.as_mut() {
            let _ = canvas.window_mut().set_title(title);
        }
    }
}

pub fn LbSetIcon(_index: i16) {
    // Icon loading from PNG/ICO is platform-specific; left as stub.
}

pub fn LbSetUserResourceMapping(f: fn(i16) -> *const libc::c_char) {
    unsafe { USER_RESOURCE_MAPPING = Some(f); }
}

pub static mut USER_RESOURCE_MAPPING: Option<fn(i16) -> *const libc::c_char> = None;

pub fn LbGraphicsScreenWidth() -> TbScreenCoord {
    unsafe { LB_DISPLAY.graphics_screen_width as i16 }
}

pub fn LbGraphicsScreenHeight() -> TbScreenCoord {
    unsafe { LB_DISPLAY.graphics_screen_height as i16 }
}
