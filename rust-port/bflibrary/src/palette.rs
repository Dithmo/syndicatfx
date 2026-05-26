// LbPalette* — palette management for 8-bit indexed mode.

use crate::types::*;
use crate::screen::LB_DISPLAY;

pub fn LbPaletteSet(palette: *const u8) -> TbResult {
    unsafe {
        LB_DISPLAY.palette = palette as *mut u8;
    }
    LB_SUCCESS
}

pub fn LbPaletteGet(palette: *mut u8) -> TbResult {
    unsafe {
        if LB_DISPLAY.palette.is_null() || palette.is_null() {
            return LB_FAIL;
        }
        std::ptr::copy_nonoverlapping(LB_DISPLAY.palette, palette, 256 * 3);
    }
    LB_SUCCESS
}

pub fn LbPaletteFade(_target: *const u8, _rate: u8, _mode: u8) -> TbResult {
    // Palette fade stub; implement with interpolation if needed.
    LB_SUCCESS
}
