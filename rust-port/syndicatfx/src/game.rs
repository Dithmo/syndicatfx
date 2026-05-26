// game.rs — mirrors game.c: initialisation, game loop, path helpers.

use bflibrary::screen::{LbBaseInitialise, LbSetTitle, LbSetIcon,
                         LbSetUserResourceMapping, LbScreenWaitVbi,
                         LbScreenReset};
use bflibrary::keyboard::{LbKeyboardClose, LB_KEY_ON, KC_F10};
use bflibrary::mouse::LbMouseReset;
use bflibrary::palette::LbPaletteSet;
use bflibrary::types::*;
use bflibrary::window::LbWindowsControl;
use bfsoundlib::audio::FreeAudio;
use crate::sound::{BFMidiStopMusic, GetMusicAble};

use crate::display::{display_lock, display_unlock, swap_wscreen, app_resource_mapping};
use crate::game_data::{GetDirectoryUser, GetDirectoryHdd, setup_file_names, free_map_level};
use crate::mouse::mouse_update_scaled_coords;
use crate::sound::{BFSonundUnkn1, BFMidiStartMusic, BFMidiIsMusicPlaying};
use crate::globals::*;
use crate::syndre::*;

use std::ffi::CStr;
use std::path::MAIN_SEPARATOR as FS_SEP;

const SAVEGAME_PATH: &str = "save/";

// ---- Initialisation -------------------------------------------------------

pub fn game_initialise() -> bool {
    let ret = LbBaseInitialise();
    if ret != LB_SUCCESS {
        eprintln!("[ERR] Bullfrog Library initialization failed");
        return false;
    }

    #[cfg(unix)]
    crate::osunix::unix_restore_signal_handlers();

    bflibrary::screen::LbSetUserResourceMapping(app_resource_mapping);
    bflibrary::screen::LbSetTitle("SyndicatFX");
    bflibrary::screen::LbSetIcon(1);

    setup_file_names();
    true
}

// ---- SDL event pump -------------------------------------------------------

pub fn game_handle_sdl_events() {
    if !LbWindowsControl() {
        game_quit();
    }
    mouse_update_scaled_coords();
}

// ---- Path helpers ---------------------------------------------------------

pub fn game_transform_path(file_name: &str) -> String {
    if file_name.to_ascii_lowercase().starts_with(SAVEGAME_PATH) {
        let rest = &file_name[SAVEGAME_PATH.len()..];
        let user = unsafe { CStr::from_ptr(GetDirectoryUser()) }
            .to_str().unwrap_or(".");
        return format!("{}{}{}", user, FS_SEP, rest);
    }
    // Absolute path
    if file_name.starts_with('/') || file_name.starts_with('\\')
        || (file_name.len() >= 2 && file_name.as_bytes()[1] == b':')
    {
        return file_name.to_string();
    }
    let hdd = unsafe { CStr::from_ptr(GetDirectoryHdd()) }
        .to_str().unwrap_or(".");
    format!("{}{}{}", hdd, FS_SEP, file_name)
}

// ---- Frame timing ---------------------------------------------------------

pub fn game_update() {
    game_update_full(true);
}

fn game_update_full(wait: bool) {
    const MAX_FPS: i32 = 16;
    const FRAME_DUR: i32 = 1000 / MAX_FPS;

    static mut LAST_FRAME_TICKS: i32 = 0;

    display_unlock();
    game_handle_sdl_events();

    let start = unsafe { sdl2::sys::SDL_GetTicks() as i32 };

    if wait {
        unsafe {
            if LAST_FRAME_TICKS != 0 {
                let elapsed = start - LAST_FRAME_TICKS + 2;
                if elapsed < FRAME_DUR {
                    let mut remaining = FRAME_DUR - elapsed;
                    let min_sleep: i32 = 1000 / 40;
                    let max_sleep: i32 = 1000 / 20;
                    let f = remaining as f32 * (min_sleep + max_sleep) as f32
                        / (2 * min_sleep * max_sleep) as f32;
                    let base = (remaining as f32 / f + 0.5) as i32;

                    while remaining > 0 {
                        let sleep = base.min(remaining) as u32;
                        let t0 = sdl2::sys::SDL_GetTicks() as i32;
                        sdl2::sys::SDL_Delay(sleep);
                        display_lock();
                        game_handle_sdl_events();
                        display_unlock();
                        remaining -= sdl2::sys::SDL_GetTicks() as i32 - t0;
                    }
                }
            }
            LAST_FRAME_TICKS = sdl2::sys::SDL_GetTicks() as i32;
        }
    }

    display_lock();
}

// ---- Shutdown -------------------------------------------------------------

pub fn reset_input() {
    LbMouseReset();
    LbKeyboardClose();
}

pub fn host_reset() {
    FreeAudio();
    reset_input();
    LbScreenReset();
}

pub fn game_quit() -> ! {
    host_reset();
    bflibrary::screen::LbBaseReset();
    std::process::exit(0);
}

// ---- Main game loop -------------------------------------------------------
//
// Direct translation of syndicate() in game.c.
// All the inner calls go to syndre.rs (the translated assembly functions).

pub fn syndicate() {
    unsafe {
        set_default_player();

        while menu_select() == 0 {
            let mut init_state: u32 = 0;

            reset_mission_info();
            initialise_player();

            while (BYTE_60AFC & 1) != 0 {
                if LB_KEY_ON[KC_F10 as usize] != 0 { break; }

                if init_state != 0 {
                    process_players_turn();
                }

                if IS_MULTIPLAYER_GAME != 0 {
                    multi_play();
                } else {
                    single_play();
                    process_computer_players();
                }

                move_it();
                correct_buttons();

                if init_state > 2 {
                    BFSonundUnkn1();
                }

                check_for_danger();
                process_day(1008);
                check_end_level();

                LbScreenWaitVbi();
                swap_screen_vres16();

                if init_state == 2 {
                    LbPaletteSet(GRAPHICS_PALETTE);
                }

                scroll_map(LB_DISPLAY_MMOUSE_X_640, LB_DISPLAY_MMOUSE_Y_400);
                scroll_map(LB_DISPLAY_MMOUSE_X_640, LB_DISPLAY_MMOUSE_Y_400);
                copy_back();
                draw_mapwho();
                draw_panel();
                click_map();
                game_update();

                if init_state < 3 { init_state += 1; }
            }

            if GetMusicAble() { BFMidiStopMusic(); }
            free_map_level();

            if LB_KEY_ON[KC_F10 as usize] == 0 {
                MOUSE_SWAP = 1;
                MOUSE_OLD_W = 0;
                transfer_people_into_player(NETWORK_SLOT);
                level_finished();
                if LB_KEY_ON[KC_F10 as usize] != 0 { break; }
            }
        }
    }
}
