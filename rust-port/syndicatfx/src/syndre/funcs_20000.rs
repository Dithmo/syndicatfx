// syndre/funcs_20000.rs — Translated x86 functions at addresses 0x20000–0x2FFFF.
//
// This range contains: initialise_player, reset_mission_info, move_it,
// single_play, multi_play, process_players_turn, correct_buttons,
// transfer_people_into_player, free_map_level, set_default_player, etc.

use crate::globals::*;
use crate::syndre::data::*;
use crate::syndre::{sar, idiv32};
use bflibrary::screen::LB_DISPLAY;

// ---- Forward declarations of untranslated functions in this range -----------

extern "C" {
    fn process_action(slot: u32);
    fn ExchangeNetwork_Packet();
    fn NetworkPlayersCount() -> u16;
    fn set_all_changes();
    fn func_4f3d2(slot: i32, font: *mut u8, row: i32, col: i32, unk: i32, unk2: i32);
    fn load_map_level(fname: *const u8, levno: u32);
    fn init_players_people(slot: u32, unk: u32);
    fn setup_panel();
    fn set_network_player(slot: u32, unk: u32);
    fn __NETCheckBios__();
    fn NetworkCmdlineSetup() -> u16;
    fn StartNetwork();
    fn ExchangeNetwork_PlayerInfo(slot: i32);
    fn StopNetwork(slot: u32);
    fn StopAllSounds();
    fn LbDataFreeAll(files: *mut u8);
    fn GetTeamMemberName() -> u8;
    fn DoResearch() -> u8;
    fn CompleteResearch();
    fn process_panel_people(panel: *mut u8, active: u32, left: u32, right: u32) -> u16;
    fn process_panel(panel: *mut u8, active: u32, left: u32, right: u32) -> u16;
    fn move_people();
    fn move_weapons();
    fn move_effects();
    fn move_objects();
    fn move_vehicles();
}

// ---- Data segment pointers used in this translation block -------------------
// Panel data blocks live at fixed addresses in the original binary.
extern "C" {
    static data_55112: u8;
    static data_55190: u8;
}

// ---------------------------------------------------------------------------
// 0x237b0  reset_mission_info
//
// Clears all per-mission state and resets flags to defaults.
// Literal translation: registers are local variables; array clear is an
// LbMemorySet call for data_60674 replaced with a safe fill.
// ---------------------------------------------------------------------------
pub fn reset_mission_info() {
    unsafe {
        // al=1, edi=7, edx=0, ah=0, ebx=0, ecx=0
        DATA_60AF0 = 0;
        DATA_60AF4 = 0;
        DATA_60AF5 = 0;
        DATA_60AF6 = 0;
        DATA_60AF7 = 0;
        DATA_60AF8 = 0;
        DATA_60AF9 = 0;
        DATA_60AFA = 0;
        BYTE_60AFC = 1;
        DATA_60AFD = 0;
        DATA_60AFE = 0;
        A11        = 0;
        DATA_60B06 = 0;
        DATA_60AFB = 0;
        SOUND_ACTIVE  = 1;
        MUSIC_ACTIVE  = 1;
        SCANNER_PULSE = 1;
        GAME_SPEED    = 7;
        DATA_60B32    = 0;

        // data_5532c = 0x0010, data_5532e = 0x0000
        DATA_5532C = 0x0010;
        DATA_5532E = 0x0000;

        // loop: data_60a7c[0..8] = 0
        for i in 0..8usize {
            DATA_60A7C[i] = 0;
        }

        // LbMemorySet(data_60674, 0, 0x408)
        for b in DATA_60674.iter_mut() { *b = 0; }
        DATA_60AE8 = 0;
    }
}

// ---------------------------------------------------------------------------
// 0x23880  set_default_player
//
// Initialises all 8 player slots: clears network data, sets up agent names,
// credits and territory states.  Register-level translation; the inner
// per-slot stride computation uses the original * 0x5c / 8 idiom.
// ---------------------------------------------------------------------------
pub fn set_default_player() {
    unsafe {
        // slot_index = 0; Network__Slot = 0
        let mut slot_idx: u16 = 0;
        NETWORK_SLOT = 0;
        let mut slot_stride8: u8 = 0; // slot_idx * 8 (0x10(%esp))

        // outer loop: for slot_idx in 0..8
        'outer: loop {
            if slot_idx >= 8 { break; }

            // stride index into stride-7 arrays: eax = slot_idx*7 (from shl 3/sub)
            let edx = slot_idx as u32;
            let mut eax = edx * 7; // slot_idx*8 - slot_idx = slot_idx*7

            // Clear network slot data
            DATA_605E0[(edx * 7) as usize] = slot_idx as u8; // type = slot index
            DATA_605DE[(edx * 7) as usize] = 0;
            PACKETS[(edx * 7) as usize]    = 0;
            DATA_605D6[(edx * 7) as usize] = 0;
            DATA_605D8[(edx * 7) as usize] = 0;
            DATA_605E1[(edx * 7) as usize] = 0;
            DATA_605DA[(edx * 7) as usize] = LEVEL_SEED;

            // Per-slot player struct offset (stride 0x5c * slot_idx after * 188/8)
            // Original: eax = slot*0x88 - slot*0x11 = slot*(0x88-0x11) computed as
            //           eax = slot*0x21*4 - slot = slot*0x83 = slot*(128+3)
            // Actual stride from asm: shl 5 + add edx + lea *4 - edx + lea *8 - edx
            //   = edx*32 + edx = edx*33; *4 = edx*132; -edx = edx*131; *8 = edx*1048; -edx = edx*1047
            // But these arrays are dimensioned [8] so index is just edx.
            let si = edx as usize;

            if slot_idx == 0 {
                DATA_5E4AD[si] = 0;
                DATA_5E4AA[si] = 1; // human player
            } else {
                DATA_5E4AD[si] = 0;
                DATA_5E4AA[si] = 2; // CPU player
            }

            // Init agent name/type arrays: 50 (0x32) agents per slot
            for agent in 0u32..0x32 {
                // data_5e555[slot_base + agent] = 0
                // data_5e587[slot_base + agent] = 0
                // In the original these are stride-1 byte arrays indexed by
                // slot*0x5c + agent; our arrays are flat [512] so index = si*0x5c + agent
                let base = si * 0x5c;
                if base + agent as usize + 1 <= 512 {
                    DATA_5E555[base + agent as usize] = 0;
                    DATA_5E587[base + agent as usize] = 0;
                }
            }

            // Set per-slot metadata
            DATA_5E4AC[si] = slot_idx as u8;
            DATA_5E4AB[si] = slot_idx as u8;
            DATA_5E4A0[si] = 0;
            DATA_5E4A4[si] = 1;
            DATA_5E4A6[si] = 0x55;

            // Set credits (cheat_credits → large amount, else 0x7530)
            PLAYERS[si] = if CHEAT_CREDITS != 0 { 0x5f5e100 } else { 0x7530 };

            // Clear misc fields
            DATA_5E552[si] = 0;
            DATA_5E4AD[si] = 0;
            DATA_5E4BF[si] = 0;
            DATA_5E4A8[si] = 0;
            DATA_5E551[si] = slot_stride8;

            // Inner loop: 0x12 (18) agents per slot, assign random names
            let mut agent_idx: u16 = 0;
            loop {
                if agent_idx >= 0x12 { break; }

                // random(ebp) → ebp was loaded from stack slot (0x3 at start of outer)
                // In context: it's random(3) for the gender seed
                // Full set_default_player uses ebp=3 (number of teams?)
                let rand_val = crate::syndre::funcs_10000::random(3);

                // Base index into per-slot agent arrays
                let base = si * 0x5c + agent_idx as usize;
                if base < 512 {
                    DATA_5E5BC[base] = (rand_val & 1) as u16;
                    DATA_5E5BA[base] = 0x10;
                    DATA_5E5B9[base] = GetTeamMemberName();
                }

                agent_idx += 1;
            }

            // Agents 18-50: mark as unused (flags = 0xFFFF, name = 0xFF)
            for extra in 0x12usize..0x32usize {
                let base = si * 0x5c + extra;
                if base < 512 {
                    DATA_5E5BA[base] = 0xffff;
                    DATA_5E5BC[base] = 0;
                    DATA_5E5B9[base] = 0xff;
                }
            }

            // Team assignments for first 4 agents
            for team in 0u8..4 {
                let agent_i = team as usize;
                let base = si * 0x5c + agent_i;
                if base < 512 {
                    DATA_5E5C0[base] = team + 1;
                }
                SELECTED_TEAM[team as usize] = team + 1;
            }

            // Remaining team slots: 0
            for team in 4usize..8 {
                let base = si * 0x5c + team;
                if base < 512 {
                    DATA_5E5C0[base] = 0;
                }
            }

            slot_stride8 = slot_stride8.wrapping_add(8);
            slot_idx += 1;
        }

        // After player init: set up territory/country data (50 countries)
        let mut country: u16 = 0;
        loop {
            if country >= 0x32 { break; }
            // Assign random territory holder or 0 if byte_60B51 is set
            // DATA_5539E is a stride-10 array of country data
            // Simplified: just advance country counter
            country += 1;
        }
    }
}

// ---------------------------------------------------------------------------
// 0x24fe0  ASM_free_map_level
//
// Frees all level file data and optionally the sound bank.
// Literal translation — the ac_LbDataFreeAll calls forward to stub impls.
// ---------------------------------------------------------------------------
pub fn ASM_free_map_level() {
    unsafe {
        // LbDataFreeAll(mission_load_files)
        if !MISSION_LOAD_FILES.is_null() {
            LbDataFreeAll(MISSION_LOAD_FILES);
        }
        // LbDataFreeAll(unkn1_empty_load_files)
        if !UNK1_EMPTY_LOAD_FILES.is_null() {
            LbDataFreeAll(UNK1_EMPTY_LOAD_FILES);
        }
        // if SoundAble: StopAllSounds; LbDataFreeAll(sound_bank_files0)
        if bfsoundlib::audio::GetSoundAble() {
            StopAllSounds();
            if !SOUND_BANK_FILES0.is_null() {
                LbDataFreeAll(SOUND_BANK_FILES0);
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 0x25560  single_play
//
// Runs one singleplayer game tick: calls process_action for each of 8 slots.
// Direct translation of the push/call/inc loop.
// ---------------------------------------------------------------------------
pub fn single_play() {
    unsafe {
        let mut ebx: u32 = 0; // slot index
        loop {
            process_action(ebx);
            ebx += 1;
            if ebx >= 8 { break; }
        }
    }
}

// ---------------------------------------------------------------------------
// 0x25580  multi_play
//
// Multiplayer game tick: sync seed/person count, exchange packets, call
// process_action for each active slot, optionally draw packet count display.
// ---------------------------------------------------------------------------
pub fn multi_play() {
    unsafe {
        // Sync seed + person count into current slot's data
        let edx = NETWORK_SLOT as u32;
        let slot7 = (edx * 7) as usize; // lea *8 - edx = edx*7

        DATA_605DA[slot7] = LEVEL_SEED;
        DATA_605DC[slot7] = LEVEL_TIMER; // level__PersonCount_UNSURE uses LEVEL_TIMER slot
        DATA_605DE[slot7] = DATA_605DE[slot7].wrapping_add(1);

        // Check if only 1 player; if so mark slot as done (0x2)
        let player_count = NetworkPlayersCount();
        if player_count == 1 {
            DATA_605E1[slot7] = 0x2;
        }

        // Exchange packets over network
        ExchangeNetwork_Packet();

        // Process actions for each slot up to NumberOfSlots
        let mut esi: u32 = 0;
        loop {
            if esi as i32 >= NETWORK_NUMBER_OF_SLOTS as i32 { break; }
            process_action(esi);
            esi += 1;
        }

        // set_all_changes to sync dirty state
        set_all_changes();
    }
}

// ---------------------------------------------------------------------------
// 0x256f0  initialise_player
//
// Sets up the player's network slot. In single-player mode this is trivial;
// in multiplayer it runs the network login sequence.
// ---------------------------------------------------------------------------
pub fn initialise_player() {
    unsafe {
        NETWORK_SLOT = 0;
        NETWORK_NUMBER_OF_SLOTS = 1;

        if IS_MULTIPLAYER_GAME != 0 {
            // Multiplayer: login and sync
            let login_result = {
                __NETCheckBios__();
                NetworkCmdlineSetup()
            };
            if login_result >= 0xfffe {
                // Login failed/aborted — exit
                std::process::exit(1);
            }
            // Set up player slot and start network
            set_network_player(NETWORK_SLOT as u32, 0);
            StartNetwork();
            let slot = NETWORK_SLOT as i32;
            ExchangeNetwork_PlayerInfo(slot);
        } else {
            // Single-player
            IS_MULTIPLAYER_GAME = 0;
            set_network_player(0, 0);
        }
    }
}

// ---------------------------------------------------------------------------
// 0x27db0  correct_buttons
//
// Clamps lbDisplay.LeftButton and RightButton:
//   > 1  → 0
//   == 1 → add 1 (becomes 2)
// Literal translation of the two identical clamp sequences.
// ---------------------------------------------------------------------------
pub fn correct_buttons() {
    unsafe {
        // LeftButton
        if LB_DISPLAY.left_button > 1 {
            LB_DISPLAY.left_button = 0;
        } else if LB_DISPLAY.left_button == 1 {
            LB_DISPLAY.left_button += 1; // 1 → 2
        }
        // RightButton
        if LB_DISPLAY.right_button > 1 {
            LB_DISPLAY.right_button = 0;
        } else if LB_DISPLAY.right_button == 1 {
            LB_DISPLAY.right_button += 1; // 1 → 2
        }
    }
}

// ---------------------------------------------------------------------------
// 0x29250  move_it
//
// Advances all entity types one simulation step.  Direct tail-call chain.
// ---------------------------------------------------------------------------
pub fn move_it() {
    unsafe {
        move_worlds();
        move_people();
        move_weapons();
        move_effects();
        move_objects();
        move_vehicles();
    }
}

// ---------------------------------------------------------------------------
// 0x2c880  process_players_turn
//
// Reads player input via process_panel_people / process_panel and queues
// agent commands.  The panel data blocks live at fixed C addresses.
// ---------------------------------------------------------------------------
pub fn process_players_turn() {
    unsafe {
        // Increment frame timer
        LEVEL_TIMER = LEVEL_TIMER.wrapping_add(1);

        // Copy DATA_5E124 → DATA_5E122 (save old active people selection)
        DATA_5E122 = DATA_5E124;

        let left  = LB_DISPLAY.left_button  as u32;
        let right = LB_DISPLAY.right_button as u32;

        // process_panel_people(data_55112, DATA_5E124, left, right)
        let active_people = DATA_5E124 as u32;
        let new_sel = process_panel_people(
            &data_55112 as *const u8 as *mut u8,
            active_people, left, right,
        );

        if new_sel != 0 {
            DATA_5E112 = 1;
            if left != 0 && new_sel as u32 == active_people {
                DATA_5E112 = 0;
            }
            DATA_5E124 = new_sel;
        }

        // process_panel(data_55190, A2, left, right)
        let a2_val = A2 as u32;
        let panel_res = process_panel(
            &data_55190 as *const u8 as *mut u8,
            a2_val, left, right,
        );

        if panel_res != 0 {
            let old_a2 = A2;
            A2 = panel_res;

            if old_a2 == 2 {
                DATA_5E11E = 2;
                A2 = 1;
            } else {
                DATA_5E11E = old_a2;
                // Iterate panel entries, set command fields
                // (requires panel struct traversal — stub for now)
            }
            DATA_5E110 = 1;
        }
    }
}

// ---------------------------------------------------------------------------
// 0x18810  process_day
//
// Handles F1-F5 key toggles (sound/music/speed/scanner), day/research
// progression, and returns non-zero when a day boundary is crossed.
// Structural translation — DoResearch remains an extern stub.
// ---------------------------------------------------------------------------
pub fn process_day(ticks: u32) -> u8 {
    unsafe {
        let mut game_spd = GAME_SPEED;

        // F1 (KC_F1 = 0x3b): toggle SoundActive (wait for release)
        if LB_KEY_ON_ARR[0x3b] != 0 {
            while LB_KEY_ON_ARR[0x3b] == 1 {
                // spin; SDL events pumped by game_update
            }
            SOUND_ACTIVE ^= 1;
        }

        // F2 (0x3c): toggle MusicActive
        if LB_KEY_ON_ARR[0x3c] != 0 {
            while LB_KEY_ON_ARR[0x3c] == 1 {}
            MUSIC_ACTIVE ^= 1;
        }

        // F3 (0x3d): decrease speed (minimum 0)
        if LB_KEY_ON_ARR[0x3d] != 0 && game_spd > 0 {
            while LB_KEY_ON_ARR[0x3d] == 1 {}
            game_spd -= 1;
        }

        // F4 (0x3e): increase speed (maximum 0xc)
        if LB_KEY_ON_ARR[0x3e] != 0 && game_spd < 0xc {
            while LB_KEY_ON_ARR[0x3e] == 1 {}
            game_spd += 1;
        }

        // F5 (0x3f): toggle ScannerPulse
        if LB_KEY_ON_ARR[0x3f] != 0 {
            while LB_KEY_ON_ARR[0x3f] == 1 {}
            SCANNER_PULSE ^= 1;
        }

        GAME_SPEED = game_spd;

        // Singleplayer day progression only
        if IS_MULTIPLAYER_GAME != 0 {
            return 0;
        }

        // Speed throttle: if game_spd >= 3, spin until DATA_60B50 catches up
        if game_spd >= 3 {
            loop {
                let b50 = DATA_60B50 as u32;
                if b50 >= game_spd { break; }
                if DATA_60B50 == 0 { break; }
            }
        }
        DATA_60B50 = 0;

        // Day counter: compute slot offset (same stride as other per-slot arrays)
        let slot = NETWORK_SLOT as u32;
        let si = slot as usize;

        // Compute research day: day_counter / (ticks / 0x18)
        let divisor = if ticks == 0 { 1 } else { ticks / 0x18 };
        let day = if divisor == 0 { 0 } else { DATA_5E4A0[si] / divisor };

        let last_day = DATA_53EE8;
        if day as u8 != last_day {
            DATA_53EE8 = day as u8;
            if RESEARCH != 0 && RESEARCH != 3 {
                // no research this day
            } else {
                let new_research = DoResearch();
                RESEARCH = new_research;
            }
        }

        // Decrement day counter; if exhausted, increment days/years
        if ticks == 0 || DATA_5E4A0[si] < ticks - 1 {
            // Day boundary crossed
            DATA_5E4A0[si] = 0x17a3; // reset to ~24h ticks
            let new_days = DATA_5E4A4[si].wrapping_add(1);
            DATA_5E4A4[si] = new_days;
            if new_days > 0x16d {
                // New year
                DATA_5E4A4[si] = 1;
                DATA_5E4A6[si] = DATA_5E4A6[si].wrapping_add(1);
            }
            return 1;
        } else {
            DATA_5E4A0[si] = DATA_5E4A0[si].wrapping_sub(1);
        }
        0
    }
}

// ---------------------------------------------------------------------------
// 0x246d0  transfer_people_into_player
//
// Iterates all entities in the player's agent list. For each entity that
// belongs to the current slot and is attached to a vehicle/crate, adds a
// credit bonus to PLAYERS[slot] based on the entity's flags and increments
// DATA_60AFD (mission tally counter). In singleplayer only.
//
// Structural translation: the per-entity flag dispatch (flags 0x1/0x8/0x4/
// 0x10/0x2) is faithful; the inner loop that scans for a free agent name
// slot is stubbed (requires full level__People struct layout).
// ---------------------------------------------------------------------------
pub fn transfer_people_into_player(slot: i16) {
    unsafe {
        if IS_MULTIPLAYER_GAME != 0 { return; }

        let si = slot as usize;
        let agent_count = DATA_5E551.get(si).copied().unwrap_or(0) as usize;
        if LEVEL_PEOPLE.is_null() { return; }

        // Iterate each agent entry for this slot
        for agent_idx in 0..agent_count {
            let base = si * 0x5c;
            // data_5e5c0[slot_base + agent_idx] = team assignment
            if base + agent_idx >= 512 { break; }
            let team = DATA_5E5C0[base + agent_idx];
            if team == 0 { continue; }

            // Compute entity pointer into level__People
            let entity_off = (DATA_5E551[si] as usize)
                .wrapping_sub(1)
                .wrapping_add(team as usize)
                .wrapping_mul(0x5c);
            let entity = LEVEL_PEOPLE.add(entity_off);

            // Check entity is alive (0xb & 0x1 == 0) and has a vehicle link
            if (*entity.add(0x0b) & 0x1) != 0 { continue; } // dead
            let vehicle_link = *(entity.add(0x20) as *const u16);
            if vehicle_link == 0 { continue; } // no vehicle

            let ch = *entity.add(0x1c); // entity flags byte

            // Dispatch on flag bits — credit amounts from original assembly
            let bonus: u32 = if (ch & 0x01) != 0 {
                0x32  // +50 credits (captured agent)
            } else if (ch & 0x08) != 0 {
                0x96  // +150 credits (equipment)
            } else if (ch & 0x04) != 0 {
                0x96
            } else if (ch & 0x10) != 0 {
                0x12c // +300 credits (vehicle)
            } else if (ch & 0x02) != 0 {
                // Variable: add entity's field_0x14 value
                let fld14 = *(entity.add(0x14) as *const i16) as i32;
                if fld14 >= 0 { fld14 as u32 } else { 0 }
            } else {
                continue;
            };

            // PLAYERS[si] += bonus; clear vehicle link; increment tally
            PLAYERS[si] = PLAYERS[si].wrapping_add(bonus);
            *(entity.add(0x20) as *mut u16) = 0;
            DATA_60AFD = DATA_60AFD.wrapping_add(1);
        }
    }
}

// ---------------------------------------------------------------------------
// set_mission_complete (called by level_complete in funcs_10000)
// ---------------------------------------------------------------------------
pub fn set_mission_complete() {
    unsafe {
        BYTE_60AFC |= 0x2;
    }
}

// ---------------------------------------------------------------------------
// 0x29270  remove_model
//
// Removes entity from the mapwho cell list, then clears its type byte (0x18).
// ---------------------------------------------------------------------------
pub fn remove_model(entity: *mut u8) {
    unsafe {
        move_off_mapwho(entity);
        *entity.add(0x18) = 0;
    }
}

// ---------------------------------------------------------------------------
// 0x29290  animate_model
//
// Advances the entity's animation frame (field 0x10 → next frame index via
// frames table, stride 8). Returns the loop flag (bit 0 of frames[new].field5).
// ---------------------------------------------------------------------------
pub fn animate_model(entity: *mut u8) -> u8 {
    use crate::globals::FRAMES;
    unsafe {
        let mut eax: u32 = 0;
        let edx = entity;
        eax = *(edx.add(0x10) as *const u16) as u32;
        let ebx = FRAMES;
        eax = eax.wrapping_mul(8);
        eax = (ebx as usize).wrapping_add(eax as usize) as u32;
        let next_frame = *(eax as *const u8).add(6) as u32
                       | ((*(eax as *const u8).add(7) as u32) << 8);
        *(edx.add(0x10) as *mut u16) = next_frame as u16;
        eax = next_frame.wrapping_mul(8);
        eax = (ebx as usize).wrapping_add(eax as usize) as u32;
        let flag = *(eax as *const u8).add(5) & 0x1;
        flag
    }
}

// ---------------------------------------------------------------------------
// 0x292d0  move_worlds
//
// Applies a small random drift to the wind angle (data_9bc78), then uses the
// 8.8 sin/cos tables to resolve the current wind speed (data_9bc77) into
// x-component (level__Worlds) and y-component (data_9bc72).
// ---------------------------------------------------------------------------
pub fn move_worlds() {
    use crate::syndre::funcs_10000::random;
    use crate::syndre::data::{DATA_5AB60, DATA_5AD60};
    unsafe {
        let r = random(9) as u8;
        DATA_9BC78 = DATA_9BC78.wrapping_add(r).wrapping_sub(2);
        let angle = DATA_9BC78 as usize;
        let speed = DATA_9BC77 as i32;
        let ebx = (DATA_5AB60[angle] as i32).wrapping_mul(speed);
        let eax = (DATA_5AD60[angle] as i32).wrapping_mul(speed);
        LEVEL_WORLDS = (sar(ebx as u32, 8) as i32) as i16;
        DATA_9BC72   = (sar(eax as u32, 8) as i32) as i16;
    }
}

// ---------------------------------------------------------------------------
// 0x29330  affect_by_wind
//
// Accumulates the current wind components into the entity-local position
// accumulators data_60b28/2a.
// ---------------------------------------------------------------------------
pub fn affect_by_wind() {
    unsafe {
        let wx = LEVEL_WORLDS as i32;
        let wy = DATA_9BC72 as i32;
        let dx = DATA_60B28 as i32;
        let bx = DATA_60B2A as i32;
        DATA_60B28 = (dx.wrapping_add(wx)) as i16;
        DATA_60B2A = (bx.wrapping_add(wy)) as i16;
    }
}

// ---------------------------------------------------------------------------
// 0x29360  getdist
//
// Returns max(abs(dx), abs(dy)) — the Chebyshev / chessboard distance.
// ---------------------------------------------------------------------------
pub fn getdist(dx: i16, dy: i16) -> i16 {
    let ax = (dx as i32).unsigned_abs();
    let bx = (dy as i32).unsigned_abs();
    if bx <= ax { ax as i16 } else { bx as i16 }
}

// ---------------------------------------------------------------------------
// 0x29390  goto_angle
//
// Adds the speed-scaled 2-D displacement for a given angle to the x/y
// accumulators (data_60b28 / data_60b2a).
//
// Arguments: speed (i16), angle (u8 index into sin/cos tables)
// ---------------------------------------------------------------------------
pub fn goto_angle(speed: i16, angle: u8) {
    use crate::syndre::data::{DATA_5AB60, DATA_5AD60};
    unsafe {
        let edx = speed as i32;
        let ebx = angle as usize;
        let ecx = (DATA_5AB60[ebx] as i32).wrapping_mul(edx);
        let eax_x = (DATA_60B28 as i32).wrapping_add(sar(ecx as u32, 8) as i32);
        DATA_60B28 = eax_x as i16;
        let eax_y = (DATA_5AD60[ebx] as i32).wrapping_mul(edx);
        let eax_y2 = (DATA_60B2A as i32).wrapping_add(sar(eax_y as u32, 8) as i32);
        DATA_60B2A = eax_y2 as i16;
    }
}

// ---------------------------------------------------------------------------
// 0x293e0  goto_zangle
//
// Like goto_angle but also updates the z accumulator (data_60b2c) using a
// second angle index.
//
// Arguments: speed (i16), xy_angle (u8), z_angle (u8)
// ---------------------------------------------------------------------------
pub fn goto_zangle(speed: i16, xy_angle: u8, z_angle: u8) {
    use crate::syndre::data::{DATA_5AB60, DATA_5AD60};
    unsafe {
        let edx = speed as i32;
        let ebx = xy_angle as usize;
        // x accumulator
        let ecx = (DATA_5AB60[ebx] as i32).wrapping_mul(edx);
        let eax_x = (DATA_60B28 as i32).wrapping_add(sar(ecx as u32, 8) as i32);
        DATA_60B28 = eax_x as i16;
        // y accumulator
        let eax_y = (DATA_5AD60[ebx] as i32).wrapping_mul(edx);
        let ebx2 = sar(eax_y as u32, 8) as i32;
        let eax_y2 = (DATA_60B2A as i32).wrapping_add(ebx2);
        DATA_60B2A = eax_y2 as i16;
        // z accumulator — uses sin table with second angle
        let za = z_angle as usize;
        let eax_z = (DATA_5AB60[za] as i32).wrapping_mul(edx);
        let eax_z2 = (DATA_60B2C as i32).wrapping_add(sar(eax_z as u32, 8) as i32);
        DATA_60B2C = eax_z2 as i16;
    }
}

// ---------------------------------------------------------------------------
// 0x29460  get_angle
//
// Computes the 256-unit angle from (0,0) toward (dx, dy).
// Thin wrapper: sign-extends both args to i32 and tail-calls arctan.
// ---------------------------------------------------------------------------
pub fn get_angle(dx: i16, dy: i16) -> u16 {
    crate::syndre::funcs_40000::arctan(dx, dy)
}

// ---------------------------------------------------------------------------
// 0x29480  goto_point
//
// If the current speed is large enough to reach (dx,dy) this tick, applies
// the full displacement to the accumulators and returns -1 (arrived).
// Otherwise computes the angle toward the target, applies one speed-scaled
// step via goto_angle, and returns the angle.
//
// Arguments: speed (i16), dx (i16), dy (i16)
// Returns:   -1 (i32) if arrived, else the angle as u16
// ---------------------------------------------------------------------------
pub fn goto_point(speed: i16, dx: i16, dy: i16) -> i32 {
    use crate::syndre::funcs_40000::arctan;
    use crate::syndre::data::{DATA_5AB60, DATA_5AD60};
    unsafe {
        let edx = dx as i32;
        let eax = dy as i32;
        let ebx = speed as i32;

        // Compare speed² against dx²+dy²
        let dist_sq = edx.wrapping_mul(edx).wrapping_add(eax.wrapping_mul(eax));
        let spd_sq  = ebx.wrapping_mul(ebx);

        if spd_sq > dist_sq {
            // Close enough — full step, return -1
            DATA_60B28 = (DATA_60B28 as i32).wrapping_add(edx) as i16;
            DATA_60B2A = (DATA_60B2A as i32).wrapping_add(eax) as i16;
            return -1_i32;
        }

        // Partial step — compute angle then apply goto_angle
        let angle = arctan(dx, dy) as usize;
        let ecx = (DATA_5AB60[angle] as i32).wrapping_mul(ebx);
        let new_x = (DATA_60B28 as i32).wrapping_add(sar(ecx as u32, 8) as i32);
        DATA_60B28 = new_x as i16;
        let edy = (DATA_5AD60[angle] as i32).wrapping_mul(ebx);
        let new_y = (DATA_60B2A as i32).wrapping_add(sar(edy as u32, 8) as i32);
        DATA_60B2A = new_y as i16;
        angle as i32
    }
}

// ---------------------------------------------------------------------------
// Helper: compute mapwho cell byte-offset from entity field_4 / field_6.
// Used by move_off_mapwho, move_on_mapwho, and move_mapwho.
//
// field_4 (i16): x position; field_4 >> 8 = tile_x (signed)
// field_6 (u16): y position; (field_6 >> 8) & 0x7F = tile_y_raw
// cell = (tile_x & 0x7F) | (tile_y_raw * 128)
// byte_offset = cell * 2
// ---------------------------------------------------------------------------
#[inline]
unsafe fn mapwho_cell_offset(entity: *const u8) -> usize {
    let f4 = *(entity.add(4) as *const i16) as i32;
    let f6 = *(entity.add(6) as *const u16);
    let tile_x = (f4 >> 8) & 0x7F;
    let tile_y = ((f6 >> 8) & 0x7F) as i32;
    ((tile_x | (tile_y << 7)) as usize) << 1
}

// ---------------------------------------------------------------------------
// 0x29660  move_off_mapwho
//
// Unlinks entity from the mapwho spatial cell it currently occupies.
// Entity fields (u16 offsets from LEVEL_THINGS_BASE):
//   field_0: next link   field_2: prev link (0 = head of list)
//   field_0xa bit 2: "on mapwho" flag
// ---------------------------------------------------------------------------
pub fn move_off_mapwho(entity: *mut u8) {
    unsafe {
        if (*entity.add(0xa) & 0x4) == 0 { return; }

        let cell_off = mapwho_cell_offset(entity);
        let mapwho_ptr = LEVEL_MAPWHO.add(cell_off) as *mut u16;

        let prev_link = *(entity.add(2) as *const u16);
        let next_link = *(entity.add(0) as *const u16);

        // Find the slot to write next_link into (either head cell or prev.field_0)
        let write_next: *mut u16 = if prev_link != 0 {
            LEVEL_THINGS_BASE.add(prev_link as usize) as *mut u16
        } else {
            mapwho_ptr
        };
        *write_next = next_link;

        // Update next entity's prev link
        if next_link != 0 {
            let next_entity = LEVEL_THINGS_BASE.add(next_link as usize);
            *(next_entity.add(2) as *mut u16) = prev_link;
        }

        *entity.add(0xa) &= !0x4;
    }
}

// ---------------------------------------------------------------------------
// 0x296d0  move_on_mapwho
//
// Links entity into the mapwho cell for its new position (field_4, field_6
// come from the arguments, not entity fields, at call time).
// Arguments: entity, new_x (i16), new_y (i16), new_z (i16)
//
// The entity is prepended to the head of the target cell's list.
// ---------------------------------------------------------------------------
pub fn move_on_mapwho(entity: *mut u8, new_x: i16, new_y: i16, new_z: i16) {
    unsafe {
        if (*entity.add(0xa) & 0x4) != 0 { return; }

        // Compute cell from the NEW coordinates being written
        let f4_val = new_x;
        let f6_val = new_y as u16;
        let tile_x = ((f4_val as i32) >> 8) & 0x7F;
        let tile_y = ((f6_val >> 8) & 0x7F) as i32;
        let cell_off = ((tile_x | (tile_y << 7)) as usize) << 1;
        let mapwho_ptr = LEVEL_MAPWHO.add(cell_off) as *mut u16;

        // Byte offset of entity from LEVEL_THINGS_BASE
        let ent_off = (entity as usize).wrapping_sub(LEVEL_THINGS_BASE as usize);

        // Prepend: entity.field_2 = 0; entity.field_0 = *mapwho_cell; *mapwho_cell = ent_off
        *(entity.add(2) as *mut u16) = 0;
        let old_head = *mapwho_ptr;
        *(entity.add(0) as *mut u16) = old_head;
        if old_head != 0 {
            let old_head_ent = LEVEL_THINGS_BASE.add(old_head as usize);
            *(old_head_ent.add(2) as *mut u16) = ent_off as u16;
        }
        *mapwho_ptr = ent_off as u16;
        *entity.add(0xa) |= 0x4;

        // Write new position fields
        *(entity.add(4) as *mut i16) = new_x;
        *(entity.add(6) as *mut i16) = new_y;
        *(entity.add(8) as *mut i16) = new_z;
    }
}

// ---------------------------------------------------------------------------
// 0x29530  move_mapwho
//
// Moves an entity from its current cell to a new one, clamping coordinates
// to stay within map bounds (0x200..0x7E00 for x, 0x200..0x5E00 for y).
// Arguments: entity, new_x (i16), new_y (i16), new_z (i16)
// ---------------------------------------------------------------------------
pub fn move_mapwho(entity: *mut u8, mut new_x: i16, mut new_y: i16, new_z: i16) {
    unsafe {
        // Clamp x to [0x200, 0x7E00)
        let tx = (new_x as i32) >> 8;
        let ty = (new_y as i32) >> 8;
        if tx < 1   { new_x = 0x7e00_u16 as i16; }
        if ty < 1   { new_y = 0x5e00_u16 as i16; }
        if tx >= 0x7f { new_x = 0x0200_u16 as i16; }
        if ty >= 0x5f { new_y = 0x0200_u16 as i16; }

        // Compute old and new cell offsets
        let old_cell = mapwho_cell_offset(entity);
        let new_tile_x = ((new_x as i32) >> 8) & 0x7F;
        let new_tile_y = (((new_y as u16) >> 8) & 0x7F) as i32;
        let new_cell = ((new_tile_x | (new_tile_y << 7)) as usize) << 1;

        if old_cell != new_cell {
            let on_flag = *entity.add(0xa) & 0x4;
            if on_flag != 0 {
                // Remove from old cell
                let old_mw = LEVEL_MAPWHO.add(old_cell) as *mut u16;
                let prev = *(entity.add(2) as *const u16);
                let next = *(entity.add(0) as *const u16);
                let write_next: *mut u16 = if prev != 0 {
                    LEVEL_THINGS_BASE.add(prev as usize) as *mut u16
                } else {
                    old_mw
                };
                *write_next = next;
                if next != 0 {
                    let ne = LEVEL_THINGS_BASE.add(next as usize);
                    *(ne.add(2) as *mut u16) = prev;
                }
                *entity.add(0xa) &= !0x4;
            }
            if (*entity.add(0xa) & 0x4) == 0 {
                // Prepend into new cell
                let new_mw = LEVEL_MAPWHO.add(new_cell) as *mut u16;
                let ent_off = (entity as usize).wrapping_sub(LEVEL_THINGS_BASE as usize);
                *(entity.add(2) as *mut u16) = 0;
                let old_head = *new_mw;
                *(entity.add(0) as *mut u16) = old_head;
                if old_head != 0 {
                    let ohe = LEVEL_THINGS_BASE.add(old_head as usize);
                    *(ohe.add(2) as *mut u16) = ent_off as u16;
                }
                *new_mw = ent_off as u16;
                *entity.add(0xa) |= 0x4;
            }
        }
        // Write new position
        *(entity.add(4) as *mut i16) = new_x;
        *(entity.add(6) as *mut i16) = new_y;
        *(entity.add(8) as *mut i16) = new_z;
    }
}
