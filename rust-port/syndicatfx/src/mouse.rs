// mouse.rs — mirrors mouse.c: coordinate scaling between screen and game space.

use bflibrary::screen::LB_DISPLAY;
use crate::globals::*;

pub fn mouse_update_scaled_coords() {
    unsafe {
        if (DRAW_FLAGS & DRW_F_SCREEN_VRES16) != 0 {
            // 640×480 mode — direct coords, subtract panel offset from Y
            LB_DISPLAY_MOUSE_X_640  = LB_DISPLAY.mouse_x as i16;
            LB_DISPLAY_MOUSE_Y_400  = (LB_DISPLAY.mouse_y - 40).max(0) as i16;
            LB_DISPLAY_MMOUSE_X_640 = LB_DISPLAY.mmouse_x as i16;
            LB_DISPLAY_MMOUSE_Y_400 = (LB_DISPLAY.mmouse_y - 40).max(0) as i16;
        } else {
            // 320×200 mode — scale up 2×
            LB_DISPLAY_MOUSE_X_640  = (LB_DISPLAY.mouse_x  * 2) as i16;
            LB_DISPLAY_MOUSE_Y_400  = (LB_DISPLAY.mouse_y  * 2) as i16;
            LB_DISPLAY_MMOUSE_X_640 = (LB_DISPLAY.mmouse_x * 2) as i16;
            LB_DISPLAY_MMOUSE_Y_400 = (LB_DISPLAY.mmouse_y * 2) as i16;
        }
        if LB_DISPLAY_MMOUSE_Y_400 > 399 {
            LB_DISPLAY_MMOUSE_Y_400 = 399;
        }
    }
}
