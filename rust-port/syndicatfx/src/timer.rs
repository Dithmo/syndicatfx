// timer.rs — mirrors timer.c: 18.2 Hz tick counter + SMACK timer.

pub fn timer_get_18_2_hz_ticks() -> u32 {
    let ms = unsafe { sdl2::sys::SDL_GetTicks() as u64 };
    ((ms * 1193182 / 1000) >> 15) as u32
}

pub fn MSSSMACKTIMERREAD() -> u32 {
    unsafe { sdl2::sys::SDL_GetTicks() }
}
