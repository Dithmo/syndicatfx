// LbWindowsControl — SDL2 event pump replacing bflibrary's event loop.

use crate::types::*;
use crate::keyboard::{lb_keyboard_update, K_ACTN_KEYDOWN, K_ACTN_KEYUP};
use crate::mouse::lb_mouse_update;
use crate::screen::SDL_CONTEXT;

// Map SDL scancode to PC scancode (simplified; extend as needed)
fn sdl_scancode_to_pc(code: sdl2::keyboard::Scancode) -> u8 {
    use sdl2::keyboard::Scancode::*;
    match code {
        Escape    => 1,  Return => 28, Space => 57, Backspace => 14, Tab => 15,
        Num1 => 2, Num2 => 3, Num3 => 4, Num4 => 5, Num5 => 6,
        Num6 => 7, Num7 => 8, Num8 => 9, Num9 => 10, Num0 => 11,
        Q => 16, W => 17, E => 18, R => 19, T => 20,
        Y => 21, U => 22, I => 23, O => 24, P => 25,
        A => 30, S => 31, D => 32, F => 33, G => 34,
        H => 35, J => 36, K => 37, L => 38,
        Z => 44, X => 45, C => 46, V => 47, B => 48, N => 49, M => 50,
        F1 => 59, F2 => 60, F3 => 61, F4 => 62, F5 => 63,
        F6 => 64, F7 => 65, F8 => 66, F9 => 67, F10 => 68,
        F11 => 87, F12 => 88,
        Left => 75, Right => 77, Up => 72, Down => 80,
        _ => 0,
    }
}

/// Process pending SDL events, update input state.  Returns `true` while the
/// application should keep running, `false` on quit request.
pub fn LbWindowsControl() -> bool {
    unsafe {
        let sdl = match SDL_CONTEXT.as_ref() {
            Some(s) => s,
            None => return true,
        };
        let mut pump = sdl.event_pump().unwrap();
        let mut quit = false;

        for event in pump.poll_iter() {
            use sdl2::event::Event;
            match event {
                Event::Quit { .. } => { quit = true; }
                Event::KeyDown { scancode: Some(sc), .. } => {
                    let pc = sdl_scancode_to_pc(sc);
                    if pc != 0 {
                        lb_keyboard_update(K_ACTN_KEYDOWN, pc);
                    }
                }
                Event::KeyUp { scancode: Some(sc), .. } => {
                    let pc = sdl_scancode_to_pc(sc);
                    if pc != 0 {
                        lb_keyboard_update(K_ACTN_KEYUP, pc);
                    }
                }
                Event::MouseMotion { x, y, xrel, yrel, mousestate, .. } => {
                    lb_mouse_update(x, y, xrel, yrel,
                        mousestate.left(), mousestate.right(), mousestate.middle());
                }
                Event::MouseButtonDown { x, y, mouse_btn, .. } => {
                    use sdl2::mouse::MouseButton;
                    let (l, r, m) = match mouse_btn {
                        MouseButton::Left   => (true,  false, false),
                        MouseButton::Right  => (false, true,  false),
                        MouseButton::Middle => (false, false, true),
                        _ => (false, false, false),
                    };
                    lb_mouse_update(x, y, 0, 0, l, r, m);
                }
                _ => {}
            }
        }

        !quit
    }
}
