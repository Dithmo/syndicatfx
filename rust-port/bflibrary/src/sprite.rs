// LbSprite* — sprite drawing into the 8-bit WScreen buffer.

use crate::types::*;
use crate::screen::LB_DISPLAY;

/// Draw a sprite at (x, y) into lbDisplay.WScreen using the game's simple
/// run-length encoded sprite format.
pub fn LbSpriteDraw(x: i32, y: i32, spr: *const TbSprite) -> TbResult {
    unsafe {
        if spr.is_null() || LB_DISPLAY.w_screen.is_null() {
            return LB_FAIL;
        }
        let spr = &*spr;
        let w = LB_DISPLAY.graphics_screen_width as i32;
        let h = LB_DISPLAY.graphics_screen_height as i32;

        // Simple transparent blit: the game uses a packed RLE sprite format.
        // Each row: sequences of [count|flags, pixels...] until count==0.
        let mut src = spr.data;
        for row in 0..(spr.s_height as i32) {
            let dy = y + row;
            loop {
                let cnt = *src as i32;
                src = src.add(1);
                if cnt == 0 { break; } // end of row
                if (cnt & 0x80) != 0 {
                    // transparent skip
                    let skip = cnt & 0x7F;
                    let _ = skip;
                } else {
                    // solid pixels
                    for col in 0..cnt {
                        let dx = x + col;
                        if dx >= 0 && dx < w && dy >= 0 && dy < h {
                            let px = *src;
                            if px != 0 {
                                *LB_DISPLAY.w_screen.add((dy * w + dx) as usize) = px;
                            }
                        }
                        src = src.add(1);
                    }
                }
            }
        }
        LB_SUCCESS
    }
}
