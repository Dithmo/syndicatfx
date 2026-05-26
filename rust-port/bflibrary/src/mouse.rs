// LbMouse* — SDL2-backed mouse management.

use crate::types::*;
use crate::screen::LB_DISPLAY;

pub static mut LB_MOUSE_WINDOW_X: i32 = 0;
pub static mut LB_MOUSE_WINDOW_Y: i32 = 0;
pub static mut LB_MOUSE_WINDOW_W: i32 = 640;
pub static mut LB_MOUSE_WINDOW_H: i32 = 480;

pub fn LbMouseSetup(sprite: *const TbSprite, w: u16, h: u16) -> TbResult {
    // Sprite cursor is set up via SDL; stub for now.
    let _ = (sprite, w, h);
    LB_SUCCESS
}

pub fn LbMouseReset() -> TbResult {
    unsafe {
        LB_DISPLAY.mouse_x = 0;
        LB_DISPLAY.mouse_y = 0;
    }
    LB_SUCCESS
}

pub fn LbMouseChangeMoveRatio(ratio_x: i32, ratio_y: i32) {
    unsafe {
        LB_DISPLAY.mouse_move_ratio_x = (ratio_x >> 8) as i16;
        LB_DISPLAY.mouse_move_ratio_y = (ratio_y >> 8) as i16;
    }
}

pub fn LbMouseSetWindow(x: i32, y: i32, w: i32, h: i32) {
    unsafe {
        LB_MOUSE_WINDOW_X = x;
        LB_MOUSE_WINDOW_Y = y;
        LB_MOUSE_WINDOW_W = w;
        LB_MOUSE_WINDOW_H = h;
    }
}

// Called by SDL event loop to update mouse position.
pub fn lb_mouse_update(x: i32, y: i32, rel_x: i32, rel_y: i32,
                       left: bool, right: bool, middle: bool) {
    unsafe {
        LB_DISPLAY.mouse_x   = x;
        LB_DISPLAY.mouse_y   = y;
        LB_DISPLAY.mmouse_x  = rel_x;
        LB_DISPLAY.mmouse_y  = rel_y;
        LB_DISPLAY.left_button   = if left   { 1 } else { 0 };
        LB_DISPLAY.right_button  = if right  { 1 } else { 0 };
        LB_DISPLAY.middle_button = if middle { 1 } else { 0 };
    }
}
