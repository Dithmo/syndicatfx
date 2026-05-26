// syndre/funcs_20000.rs — Translated x86 functions at addresses 0x20000–0x2FFFF.
//
// This range contains: initialise_player, reset_mission_info, move_it,
// single_play, multi_play, process_players_turn, correct_buttons,
// transfer_people_into_player, free_map_level, set_default_player, etc.

use crate::globals::*;
use crate::syndre::data::*;
use crate::syndre::{sar, idiv32};

// ---- Stubs for functions in this range ------------------------------------
// Each stub will be expanded as the full translation is done.

pub fn set_default_player_impl() {
    // 0x23880 — sets up the default player slot from Network__Slot
    unsafe {
        // Placeholder: ensure NETWORK_SLOT is in bounds
        if NETWORK_SLOT < 0 || NETWORK_SLOT > 7 {
            NETWORK_SLOT = 0;
        }
    }
}

pub fn reset_mission_info_impl() {
    // 0x237b0 — clears mission state variables
    unsafe {
        DATA_5C354 = 0;
        DATA_5C34C = 0;
        DATA_5532C = 0;
    }
}

pub fn initialise_player_impl() {
    // 0x256f0 — initialises player agent structures for the current mission
}

pub fn process_players_turn_impl() {
    // 0x2c880 — processes player input and queues agent commands
}

pub fn multi_play_impl() {
    // 0x25580 — runs one multiplayer game tick
}

pub fn single_play_impl() {
    // 0x25560 — runs one singleplayer game tick
}

pub fn move_it_impl() {
    // 0x29250 — moves all entities one step
}

pub fn correct_buttons_impl() -> u8 {
    // 0x27db0 — corrects UI button states
    0
}

pub fn set_mission_complete_impl() {
    // Called by level_complete in funcs_10000
    unsafe {
        BYTE_60AFC |= 0x2;
    }
}
