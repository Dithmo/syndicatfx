// LbTimer* — millisecond timer using SDL.

use crate::types::*;

pub fn LbTimerClock() -> TbClockMSec {
    // Return milliseconds since SDL init
    unsafe {
        if crate::screen::SDL_CONTEXT.is_some() {
            sdl2::sys::SDL_GetTicks() as TbClockMSec
        } else {
            0
        }
    }
}

pub fn LbSleepUntil(end_ticks: TbClockMSec) {
    let now = LbTimerClock();
    if end_ticks > now {
        let delay = (end_ticks - now) as u32;
        unsafe { sdl2::sys::SDL_Delay(delay); }
    }
}

pub fn LbSleepFor(ms: u32) {
    unsafe { sdl2::sys::SDL_Delay(ms); }
}
