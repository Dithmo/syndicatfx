// syndre/funcs_30000.rs — Translated x86 functions at addresses 0x30000–0x3FFFF.
//
// This range contains: menu_select (0x37600), level_finished (0x37f80),
// init_sound (0x385c0), and supporting UI + animation functions.

use crate::globals::*;
use crate::syndre::data::*;
use crate::syndre::funcs_40000::get_angle;

// Untranslated functions called from this range
extern "C" {
    fn ac_LbDataLoadAll(files: *mut u8) -> i32;
    fn ac_sound_bank_setup();
    fn weapons_in_weight(entity: *const u8) -> i32;
    fn ac_abs(x: i32) -> i32;
    fn can_i_persuad_you(entity: *mut u8, target: *mut u8) -> u16;
    fn i_cant_walk_in_this_direction(entity: *mut u8, direction: u32) -> u16;
    fn person_goto_in_vehicle(entity: *mut u8, vehicle: *mut u8);
    fn person_goto(entity: *mut u8);
    fn person_colide(entity: *mut u8) -> u16;
    fn decide_on_hug_direction(entity: *mut u8, speed: u32);
    fn drop_weapon(entity: *mut u8);
    fn do_a_hug(entity: *mut u8, dir: i32) -> u16;
    fn choose_best_weapon(entity: *mut u8, arg2: i32) -> u16;
    fn random(max: i32) -> i32;
    fn drop_all_weapons(entity: *mut u8);
    fn kill_all_weapons(entity: *mut u8);
    fn person_use_weapon(entity: *mut u8, tx: i32, ty: i32, tz: i32);
    fn bump_person(entity: *mut u8, tx: i32, ty: i32, tz: i32,
                   range1: i32, range2: i32, arg7: i32) -> *mut u8;
    fn i_can_see_and_shoot_person(entity: *mut u8, target: *mut u8, range: i32) -> *mut u8;
    fn i_can_see_and_shoot_vehicle(entity: *mut u8, target: *mut u8, range: i32) -> *mut u8;
    // move_people helpers
    fn person_intel();
    fn person_on_block(entity: *mut u8);
    fn adjust_bar_levels(entity: *mut u8);
    fn person_danger(entity: *mut u8);
    fn weapon_is_empty(entity: *mut u8) -> u16;
    fn which_frame_person(entity: *mut u8);
    // FSM helpers still in C
    fn check_for_on_coming_cars(x: i32, y: i32, z: i32) -> u16;
    fn there_is_a_road_here(x: i32, y: i32, z: i32) -> u16;
    fn auto_weapon(entity: *mut u8) -> u16;
}

// ---------------------------------------------------------------------------
// 0x37600  menu_select
//
// Loads menu sprites, sets up screen mode, plays intro/title animations,
// runs the mission briefing, and dispatches to map_selection or
// mission_briefing. Returns the next game state (0 = play, non-0 = exit).
//
// Structural translation: the sprite setup and sound loading are faithful;
// the briefing/selection UI loop is stubbed (returns 0 = "mission accepted").
// ---------------------------------------------------------------------------
pub fn menu_select_impl() -> u8 {
    unsafe {
        // Load unk984 sprite bank (menu graphics)
        if !UNK984_LOAD_FILES.is_null() {
            ac_LbDataLoadAll(UNK984_LOAD_FILES);
        }

        // Sound bank for menu music
        if bfsoundlib::audio::GetSoundAble() {
            if !SOUND_BANK_FILES0.is_null() {
                ac_LbDataLoadAll(SOUND_BANK_FILES0);
                ac_sound_bank_setup();
            }
        }

        // Set draw flags for MCGA (320×200 8bpp) menu mode
        DRAW_FLAGS = DRW_F_SCREEN_MCGA;
        MOUSE_SWAP = 1;
        MOUSE_OLD_W = 0;

        // The full implementation runs mission briefing / map selection UI.
        // Stub: return 0 (proceed to mission) to allow game loop testing.
        0
    }
}

// ---------------------------------------------------------------------------
// 0x37f80  level_finished
//
// Post-mission sequence: screen setup, country state update, win/lose
// animation, optional endgame when all 50 countries are conquered.
//
// Structural translation: flag-based dispatch and country update logic
// captured; AniPlay calls are stubbed (extern unresolved until translated).
// ---------------------------------------------------------------------------
pub fn level_finished_impl() {
    unsafe {
        SOUND_ACTIVE = 1;
        MUSIC_ACTIVE = 1;

        let byte_afc = BYTE_60AFC;

        // If player quit (ESC), skip post-mission sequence
        if (byte_afc & 0x8) != 0 {
            MOUSE_SWAP  = 1;
            MOUSE_OLD_W = 0;
            return;
        }

        // Load sound bank for results sequence
        if bfsoundlib::audio::GetSoundAble() {
            if !SOUND_BANK_FILES0.is_null() {
                ac_LbDataLoadAll(SOUND_BANK_FILES0);
                ac_sound_bank_setup();
            }
        }

        if (byte_afc & 0x2) != 0 {
            // Mission succeeded: update country ownership for current level
            // and check win condition (all 50 countries conquered).
            // Full traversal of data_5539e / country_states is stubbed.
            // Mark post-mission done.
            BYTE_60AFC |= 0x10;
        }

        if (byte_afc & 0x4) != 0 {
            // Mission failed: check agent roster for survivors.
            // If any agents remain (DATA_5E5BA[slot][agent] > 0) → show lose-anim.
            // Otherwise → play game-over sequence + set_default_player.
            let si = NETWORK_SLOT as usize;
            let mut has_agents = false;
            for agent in 0..0x12usize {
                let base = si * 0x5c + agent;
                if base < 512 && DATA_5E5BA[base] > 0 {
                    has_agents = true;
                    break;
                }
            }
            if !has_agents {
                BYTE_60AFC |= 0x10;
                crate::syndre::funcs_20000::set_default_player();
            }
        }

        MOUSE_SWAP  = 1;
        MOUSE_OLD_W = 0;
    }
}

// ---------------------------------------------------------------------------
// Shared helper for get_person_perception/intelligence/adrenlin.
//
// Computes a stat modifier scaled by `modifier`:
//   diff = current - base
//   if diff < 0: result = diff * modifier / base
//   if diff > 0: result = diff * modifier / (0xff - base)
//   if diff == 0: result = 0
//
// Literal: reproduces the sign-extend + idiv pattern from the assembly.
// ---------------------------------------------------------------------------
#[inline]
fn stat_scaled(base: u8, current: u8, modifier: i32) -> i32 {
    let diff = current as i16 - base as i16;
    if diff == 0 { return 0; }
    if diff < 0 {
        let num = (diff as i32).wrapping_mul(modifier);
        num / (base as i32)
    } else {
        let num = (diff as i32).wrapping_mul(modifier);
        num / (0xff - base as i32)
    }
}

// ---------------------------------------------------------------------------
// 0x30060  get_person_perception
//
// Returns a scaled modifier based on the difference between the person's
// current perception (field_0x51) and base perception (field_0x50).
// arg1 = entity ptr, arg2 = modifier value (scaled denominator).
// ---------------------------------------------------------------------------
pub fn get_person_perception(entity: *const u8, modifier: i32) -> i32 {
    unsafe {
        let base    = *entity.add(0x50);
        let current = *entity.add(0x51);
        stat_scaled(base, current, modifier)
    }
}

// ---------------------------------------------------------------------------
// 0x300c0  get_person_intelligence
//
// Same algorithm as get_person_perception; uses field_0x4c (base)
// and field_0x4d (current) for the intelligence stat.
// ---------------------------------------------------------------------------
pub fn get_person_intelligence(entity: *const u8, modifier: i32) -> i32 {
    unsafe {
        let base    = *entity.add(0x4c);
        let current = *entity.add(0x4d);
        stat_scaled(base, current, modifier)
    }
}

// ---------------------------------------------------------------------------
// 0x30120  get_person_adrenlin
//
// Same algorithm; uses field_0x48 (base) and field_0x49 (current).
// ---------------------------------------------------------------------------
pub fn get_person_adrenlin(entity: *const u8, modifier: i32) -> i32 {
    unsafe {
        let base    = *entity.add(0x48);
        let current = *entity.add(0x49);
        stat_scaled(base, current, modifier)
    }
}

// ---------------------------------------------------------------------------
// 0x30180  get_person_speed
//
// Computes movement speed for a person. If not in a vehicle (field_0x1c
// lacks bits 0x1002), speed is derived from the vehicle speed tier
// (field_0x3c bits 3-4: 0..3, mapped to 1000/1625/2250/2875), reduced by
// weapons weight, scaled by adrenalin. Clamps to minimum 0x10 when on foot,
// 0x0c when in vehicle.
//
// Literal: reproduces the multiply-by-625 speed table and the negative-edi
// weight penalty branch faithfully.
// ---------------------------------------------------------------------------
pub fn get_person_speed(entity: *const u8, modifier: i32) -> i32 {
    use crate::syndre::idiv32;
    unsafe {
        let field_1c = *(entity.add(0x1c) as *const u16);
        if (field_1c & 0x1002) != 0 {
            // In vehicle: just clamp adrenalin result to 0x0c
            let result = get_person_adrenlin(entity, modifier);
            return if result < 0x0c { 0x0c } else { result };
        }
        // On foot: compute base speed from vehicle-speed tier
        let field_3c = *(entity.add(0x3c) as *const u8);
        let v = ((field_3c & 0x18) >> 3) as i32; // 0..3
        let base_speed = v * 625 + 1000;
        let weight_penalty = weapons_in_weight(entity);
        let edi = base_speed - weight_penalty;
        let speed = if edi < 0 {
            // Overloaded: reduce modifier proportionally
            let abs_edi = ac_abs(edi);
            let num = (1000 - abs_edi) * modifier;
            let (quot, _) = idiv32(
                ((num as i64) >> 32) as u32,
                num as u32,
                1000u32,
            );
            let result = quot as i32;
            if result < 0x10 { 0x10 } else { result }
        } else {
            get_person_adrenlin(entity, edi)
        };
        // Clamp to minimum 0x10
        let result = get_person_adrenlin(entity, speed);
        if result < 0x10 { 0x10 } else { result }
    }
}

// ---------------------------------------------------------------------------
// 0x30250  new_state_person
//
// Decides the next FSM state for a person and writes it to field_0x19.
// If the desired state (field_0x58) is 0 or equal to the current state,
// the default is derived from context:
//   - field_0x20 != 0 → 0x1e (FOLLOW)
//   - field_0xb & 0x10 → 0x1d (GUARD)
//   - field_0x28 != 0 → 0x01 (GOTO_POINT)
//   - else           → 0x00 (STAND)
// Otherwise the non-zero desired state is used directly.
//
// Literal: direct translation of the branch chain.
// ---------------------------------------------------------------------------
pub fn new_state_person(entity: *mut u8) {
    unsafe {
        let current_state = *entity.add(0x19);
        let desired_state = *entity.add(0x58);
        if current_state == desired_state || desired_state == 0 {
            // Pick default state from context
            let new_state: u8 = if *(entity.add(0x20) as *const u16) != 0 {
                0x1e // FOLLOW
            } else if (*entity.add(0xb) & 0x10) != 0 {
                0x1d // GUARD
            } else if *(entity.add(0x28) as *const u16) != 0 {
                0x01 // GOTO_POINT
            } else {
                0x00 // STAND
            };
            *entity.add(0x58) = new_state;
        }
        *entity.add(0x19) = *entity.add(0x58);
    }
}

// ---------------------------------------------------------------------------
// 0x30600  affect_person
//
// Reads the 16-bit effect bitfield at entity.field_0xc (cleared on read),
// applies the first matching condition, and returns an action code:
//   0x80: on-fire  → set field_0xa|=8, return 0x15
//   0x08: on-fire2 → set field_0xa|=8, return 0x14
//   0x40: irradiated → return 0x12
//   0x01: shocked   → return 0x13
//   0x10: hit       → return 0x11
//   0x0400 (dh bit2): persuasion attempt — calls can_i_persuad_you;
//          if succeeds: set follow-target/state, play sound, return 0x2c
//   else: return current state (field_0x19)
// If field_0xc == 0: return current state.
//
// Literal: bit-priority dispatch exactly as in assembly.
// ---------------------------------------------------------------------------
pub fn affect_person(entity: *mut u8) -> u32 {
    use crate::syndre::data::LEVEL_THINGS_BASE;
    use crate::syndre::funcs_10000::play_distance_sample;
    unsafe {
        let fx = *(entity.add(0xc) as *const u16);
        if fx == 0 {
            return *entity.add(0x19) as u32;
        }
        *(entity.add(0xc) as *mut u16) = 0;
        let dl = (fx & 0xff) as u8;
        let dh = ((fx >> 8) & 0xff) as u8;

        if (dl & 0x80) != 0 {
            *entity.add(0xa) |= 0x8;
            return 0x15;
        }
        if (dl & 0x08) != 0 {
            *entity.add(0xa) |= 0x8;
            return 0x14;
        }
        if (dl & 0x40) != 0 {
            *entity.add(0xa) |= 0x8;
            return 0x12;
        }
        if (dl & 0x01) != 0 {
            *entity.add(0xa) |= 0x8;
            return 0x13;
        }
        if (dl & 0x10) != 0 {
            *entity.add(0xa) |= 0x8;
            return 0x11;
        }
        if (dh & 0x04) != 0 {
            // Persuasion: field_0x16 is byte offset of persuader from LEVEL_THINGS_BASE
            let target_off = *(entity.add(0x16) as *const u16) as usize;
            let target = LEVEL_THINGS_BASE.add(target_off);
            if can_i_persuad_you(entity, target) != 0 {
                *entity.add(0xa) |= 0x8;
                *entity.add(0x58) = 0x1e; // FOLLOW
                let target_link = *(entity.add(0x16) as *const u16);
                *(entity.add(0x20) as *mut u16) = target_link;
                play_distance_sample(entity, 0xe);
                return 0x2c;
            }
        }
        *entity.add(0x19) as u32
    }
}

// ---------------------------------------------------------------------------
// 0x306d0  quick_decide_on_hug_direction
//
// Picks a wall-hugging direction. Starting from (current_angle rounded to
// 90°) - 90°, rotates by +90° until i_cant_walk_in_this_direction returns
// non-zero (blocked), meaning a wall was found, or after 4 tries.
// Then computes distance to the hug target, decides LEFT (0x0e) or RIGHT
// (0x0f) by comparing try_angle to current facing, and writes the
// perpendicular walking angle into field_0x1a.
// arg2 is stored in entity.field_0x42 (speed/reference value).
//
// Literal: exact assembly control flow including the non-cardinal early-exit.
// ---------------------------------------------------------------------------
pub fn quick_decide_on_hug_direction(entity: *mut u8, arg2: u32) {
    use crate::syndre::funcs_10000::getrdist;
    unsafe {
        // Round field_0x1a to nearest 90° multiple, then step back by 90°
        let raw_angle = *entity.add(0x1a);
        let rounded = raw_angle.wrapping_add(0x20) & 0xc0; // & !0x3f rounded
        let mut try_angle = rounded.wrapping_sub(0x40);

        // Try up to 4 quarter-turns; exit on first BLOCKED direction
        for _ in 0..4 {
            if i_cant_walk_in_this_direction(entity, try_angle as u32) != 0 {
                break; // found a wall
            }
            try_angle = try_angle.wrapping_add(0x40);
        }

        // Compute distance to hug target (field_0x2e=target_x, field_0x30=target_z)
        let target_x = *(entity.add(0x2e) as *const i16);
        let target_z = *(entity.add(0x30) as *const i16);
        let cur_x    = *(entity.add(0x4)  as *const i16);
        let cur_z    = *(entity.add(0x6)  as *const i16);
        let dist = getrdist(target_x.wrapping_sub(cur_z) as i16,
                            target_z.wrapping_sub(cur_x) as i16) as u16;
        *entity.add(0x59) = 4;
        *(entity.add(0x1e) as *mut u16) = dist;
        *entity.add(0x5a) = try_angle;
        *(entity.add(0x42) as *mut u16) = arg2 as u16;

        // Direction state: only set for 0x00/0x40/0x80/0xc0 (cardinal angles)
        let dl = try_angle;
        let is_cardinal = dl == 0x00 || dl == 0x40 || dl == 0x80 || dl == 0xc0;
        if is_cardinal {
            let diff = dl as i32 - *entity.add(0x1a) as i32;
            if diff < 0 {
                // HUG_LEFT: walk at try_angle + 90°
                *entity.add(0x19) = 0x0e;
                *entity.add(0x1a) = dl.wrapping_add(0x40);
            } else {
                // HUG_RIGHT: walk at try_angle - 90°
                *entity.add(0x19) = 0x0f;
                *entity.add(0x1a) = dl.wrapping_sub(0x40);
            }
        }
        // non-cardinal: no state change, fall through
    }
}

// ---------------------------------------------------------------------------
// 0x31f60  fn_S_PERSON_STAND
//
// Idle-stand state handler. Clears the animation-done flag (field_0x54),
// clears the moving bit (field_0xa bit 3), runs one animation frame, then
// calls affect_person to apply any queued effects and update field_0x19.
//
// Literal: three-instruction prologue + two calls.
// ---------------------------------------------------------------------------
pub fn fn_s_person_stand(entity: *mut u8) {
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        *entity.add(0x54) = 0;
        *entity.add(0xa) &= !0x8;
        animate_model(entity);
        let new_state = affect_person(entity) as u8;
        *entity.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x31fc0  fn_S_PERSON_NEXT_COMMAND
//
// Dequeues the next agent command from the command list and dispatches to
// the appropriate state handler. Command types 1–11 map to FSM states;
// unknown types fall through. At the end, advances field_0x26 to cmd[0]
// (next command link) or entity.field_0x28 if none.
//
// Command block layout (at LEVEL_COMMANDS + entity.field_0x26):
//   [0..1] = u16 next-command link (0 = end of list)
//   [2..3] = u16 target entity id / vehicle id
//   [4]    = u8 target x (tile coord, scaled)
//   [5]    = u8 target y (tile coord, scaled)
//   [6]    = u8 target z (tile coord, scaled)
//   [7]    = u8 command type (1-based)
//
// Literal: jump-table body reproduced as match; DATA_60B4E pauses dispatch.
// ---------------------------------------------------------------------------
pub fn fn_s_person_next_command(entity: *mut u8) {
    use crate::syndre::data::{LEVEL_COMMANDS, DATA_60B4E};
    use crate::syndre::funcs_10000::{level_failed, level_complete};
    unsafe {
        *entity.add(0x54) = 0;
        let cmd_link = *(entity.add(0x26) as *const u16);
        if cmd_link == 0 || DATA_60B4E != 0 { return; }

        let cmd = LEVEL_COMMANDS.add(cmd_link as usize);
        let cmd_type = (*cmd.add(7)).wrapping_sub(1);
        if cmd_type > 0x0a { /* unknown: skip to advance step */ }
        else {
            match cmd_type {
                0 => {
                    // GOTO_POINT: set target coords scaled by 0x80
                    *entity.add(0x19) = 0x3;
                    *entity.add(0x58) = 0x3;
                    let tx = (*cmd.add(4) as u32) << 7 | 0x40;
                    let ty = (*cmd.add(5) as u32) << 7 | 0x40;
                    let tz = (*cmd.add(6) as u32) << 7;
                    *(entity.add(0x2e) as *mut u16) = tx as u16;
                    *(entity.add(0x30) as *mut u16) = ty as u16;
                    *(entity.add(0x32) as *mut u16) = tz as u16;
                }
                1 => {
                    // GOTO_VEHICLE (board): if not already in vehicle
                    if *(entity.add(0x24) as *const u16) == 0 {
                        *entity.add(0x19) = 0x5;
                        *entity.add(0x58) = 0x6;
                        *(entity.add(0x2c) as *mut u16) = *(cmd.add(2) as *const u16);
                    }
                }
                2 => {
                    // GUARD
                    *entity.add(0x19) = 0x7;
                    *entity.add(0x58) = 0x7;
                }
                3 => {
                    // GOTO_VEHICLE2
                    *entity.add(0x19) = 0xc;
                    *entity.add(0x58) = 0xc;
                    *(entity.add(0x2c) as *mut u16) = *(cmd.add(2) as *const u16);
                }
                4 => {
                    // EXIT_VEHICLE
                    *entity.add(0x19) = 0xd;
                    *entity.add(0x58) = 0xd;
                    *(entity.add(0x2c) as *mut u16) = *(cmd.add(2) as *const u16);
                }
                5 => {
                    // GOTO_STRUCTURE
                    *entity.add(0x19) = 0x5;
                    *entity.add(0x58) = 0x5;
                    *(entity.add(0x2c) as *mut u16) = *(cmd.add(2) as *const u16);
                }
                6 => {
                    // MISSION_FAILED
                    *entity.add(0x19) = 0x0;
                    *entity.add(0x58) = 0x0;
                    level_failed();
                }
                7 => {
                    // GOTO_POINT_PRECISE (coords / 2, not tile-scaled)
                    *entity.add(0x19) = 0x28;
                    *entity.add(0x58) = 0x28;
                    let tx = (*cmd.add(4) as i32) >> 1;
                    let ty = (*cmd.add(5) as i32) >> 1;
                    let tz = (*cmd.add(6) as u32) << 7;
                    *(entity.add(0x2e) as *mut i16) = tx as i16;
                    *(entity.add(0x30) as *mut i16) = ty as i16;
                    *(entity.add(0x32) as *mut u16) = tz as u16;
                }
                8 => {
                    // GUARD_AREA: clear command queue, switch to guard
                    *(entity.add(0x26) as *mut u16) = 0;
                    *entity.add(0x19) = 0x1d;
                }
                9 => {
                    // RESEARCH_PICKUP
                    *(entity.add(0x2c) as *mut u16) = 0x32;
                    *entity.add(0x19) = 0x2a;
                }
                10 => {
                    // MISSION_COMPLETE
                    *entity.add(0x19) = 0x0;
                    *entity.add(0x58) = 0x0;
                    level_complete();
                }
                _ => unreachable!(),
            }
        }
        // Advance command queue
        let next_link = *(cmd as *const u16);
        if next_link != 0 {
            *(entity.add(0x26) as *mut u16) = next_link;
        } else {
            *(entity.add(0x26) as *mut u16) = *(entity.add(0x28) as *const u16);
        }
    }
}

// ---------------------------------------------------------------------------
// 0x32130  fn_S_PERSON_GOTO_POINT
//
// Walk-to-point state handler. If the agent is riding a vehicle (field_0x24
// != 0 and vehicle field_0x18 == 2), delegates to person_goto_in_vehicle
// and then calls move_mapwho with the data_60b28/2a/2c displacement.
// Otherwise: person_goto, collision check, optional decide_on_hug_direction
// if blocked, then affect_person and animate_model.
//
// Literal: reproduces the vehicle branch and the normal walk branch.
// ---------------------------------------------------------------------------
pub fn fn_s_person_goto_point(entity: *mut u8) {
    use crate::syndre::data::{LEVEL_THINGS_BASE, DATA_60B28, DATA_60B2A, DATA_60B2C};
    use crate::syndre::funcs_20000::{move_mapwho, animate_model};
    unsafe {
        let vehicle_link = *(entity.add(0x24) as *const u16);
        if vehicle_link != 0 {
            let vehicle = LEVEL_THINGS_BASE.add(vehicle_link as usize);
            if *vehicle.add(0x18) == 2 {
                person_goto_in_vehicle(entity, vehicle);
                move_mapwho(entity,
                    DATA_60B28 as i16, DATA_60B2A as i16, DATA_60B2C as i16);
                return;
            }
        }
        // Normal walk
        person_goto(entity);
        if person_colide(entity) != 0 {
            decide_on_hug_direction(entity, 0x1f4);
        }
        let new_state = affect_person(entity) as u8;
        *entity.add(0x19) = new_state;
        animate_model(entity);
    }
}

// ---------------------------------------------------------------------------
// 0x315e0  who_shot_me
//
// Called when entity (victim) is hit; updates kill-event counters and sets
// aggro bits on the attacker.  victim.field_0x16 holds the attacker's word
// offset from LEVEL_THINGS_BASE.
//
// Counter semantics (which player's agents shot whom):
//   DATA_60AF4: our agent shot a player-controlled person (bit 0x1)
//   DATA_60AF5: our agent shot a person with bit 0x4
//   DATA_60AF6: our agent shot a person with bit 0x8
//   DATA_60AF7: our agent shot a person with bit 0x10 (civilian)
//   DATA_60AF8: our agent shot an enemy person (bit 0x2, outside our range)
//   DATA_60AFA: our agent shot one of our own agents (bit 0x2, inside range)
//
// Local-player range: DATA_5E551[slot*1047] gives the starting agent index N
// in LEVEL_PEOPLE; range is [People+N*0x5c, People+(N+4)*0x5c).
//
// Aggro: if attacker is player/enemy/civilian (bits 0x1/0x2/0x10), set
// attacker.field_0x1d |= 0x2 (when victim has 0x4 and attacker doesn't),
// else attacker.field_0x1c |= 0x40.
//
// Literal translation of 0x315e0–0x316c7.
// ---------------------------------------------------------------------------
pub fn who_shot_me(victim: *mut u8) {
    use crate::globals::NETWORK_SLOT;
    use crate::syndre::data::{LEVEL_THINGS_BASE, LEVEL_PEOPLE, DATA_5E551,
                               DATA_60AF4, DATA_60AF5, DATA_60AF6, DATA_60AF7,
                               DATA_60AF8, DATA_60AFA};
    unsafe {
        let attacker_word = *(victim.add(0x16) as *const u16);
        if attacker_word == 0 { return; }

        // Compute slot * 1047 byte-offset into per-slot struct arrays
        let ecx = NETWORK_SLOT as i32;  // signed extend
        let mut eax = ecx;
        eax <<= 5;           // eax = slot * 32
        eax += ecx;          // eax = slot * 33
        eax *= 4;            // eax = slot * 132
        eax -= ecx;          // eax = slot * 131
        eax *= 8;            // eax = slot * 1048
        eax -= ecx;          // eax = slot * 1047
        let slot_idx = eax as usize;

        // Starting agent index for our slot, then compute people range
        let start_n = DATA_5E551[slot_idx] as usize;
        let player_start = LEVEL_PEOPLE.add(start_n.wrapping_mul(0x5c));
        let player_end   = LEVEL_PEOPLE.add((start_n.wrapping_add(4)).wrapping_mul(0x5c));

        // Attacker pointer
        let edx = LEVEL_THINGS_BASE.add(attacker_word as usize);

        // Is the attacker one of our 4 agents?
        if edx >= player_start && edx < player_end {
            // Yes — update kill counter based on victim type
            if (*victim.add(0x1c) & 0x1) != 0 {
                DATA_60AF4 = DATA_60AF4.wrapping_add(1);
            } else if (*victim.add(0x1c) & 0x2) != 0 {
                // Is the victim also one of our agents?
                if victim >= player_start && victim < player_end {
                    DATA_60AFA = DATA_60AFA.wrapping_add(1);
                } else {
                    DATA_60AF8 = DATA_60AF8.wrapping_add(1);
                }
            } else {
                // Read victim field_0x1c into ah-position (byte)
                let ah = *victim.add(0x1c);
                if (ah & 0x10) != 0 {
                    DATA_60AF7 = DATA_60AF7.wrapping_add(1);
                } else if (ah & 0x4) != 0 {
                    DATA_60AF5 = DATA_60AF5.wrapping_add(1);
                } else if (ah & 0x8) != 0 {
                    DATA_60AF6 = DATA_60AF6.wrapping_add(1);
                }
            }
        }

        // Aggro: if attacker is a player/enemy/civilian, mark it hostile
        let cl = *edx.add(0x1c);
        if (cl & 0x1) != 0 || (cl & 0x2) != 0 || (cl & 0x10) != 0 {
            if (*victim.add(0x1c) & 0x4) != 0 && (*edx.add(0x1c) & 0x4) == 0 {
                *edx.add(0x1d) |= 0x2;
            } else {
                *edx.add(0x1c) |= 0x40;
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 0x321c0  fn_S_PERSON_GOTO_STRUCTURE
//
// Walk to a target structure or vehicle. If the target entity is dead (field_0xb
// bit 0), relinquish control via new_state_person. If the agent is following
// a vehicle (field_0x58 == 6, follow-target without its own vehicle), also
// relinquish. Otherwise copy target's xyz into entity.field_0x2e/30/32 and
// either finish via new_state_person (arrived), delegate to person_goto_in_vehicle,
// or walk/hug-wall on foot.
//
// Literal translation of 0x321c0–0x322f3.
// ---------------------------------------------------------------------------
pub fn fn_s_person_goto_structure(entity: *mut u8) {
    use crate::syndre::data::{LEVEL_THINGS_BASE, DATA_60B28, DATA_60B2A, DATA_60B2C};
    use crate::syndre::funcs_20000::{move_mapwho, animate_model, getdist};
    unsafe {
        let eax_link = *(entity.add(0x2c) as *const u16) as usize;
        let target = LEVEL_THINGS_BASE.add(eax_link);

        if (*target.add(0x0b) & 0x1) != 0 {
            let al = *entity.add(0x19);
            *entity.add(0x58) = al;
            new_state_person(entity);
            return;
        }

        let follow_link = *(entity.add(0x20) as *const u16);
        if follow_link != 0 && *entity.add(0x58) == 6 {
            let follow = LEVEL_THINGS_BASE.add(follow_link as usize);
            if *(follow.add(0x24) as *const u16) == 0 {
                let al = *entity.add(0x19);
                *entity.add(0x58) = al;
                new_state_person(entity);
                return;
            }
        }

        // Copy target x/y/z to entity goto fields
        let tx = *(target.add(0x4) as *const u16);
        let ty = *(target.add(0x6) as *const u16);
        let tz = *(target.add(0x8) as *const u16);
        *(entity.add(0x2e) as *mut u16) = tx;
        *(entity.add(0x30) as *mut u16) = ty;
        *(entity.add(0x32) as *mut u16) = tz;

        if *target.add(0x18) == 2 {
            // Target is a vehicle — check proximity to camera position
            let dx = (*(entity.add(0x2e) as *const i16) as i32
                      - DATA_60B28 as i32) as i16;
            let dy = (*(entity.add(0x30) as *const i16) as i32
                      - DATA_60B2A as i32) as i16;
            if getdist(dx, dy) < 0x80 {
                new_state_person(entity);
                return;
            }
        }

        // In vehicle?
        let veh_link = *(entity.add(0x24) as *const u16);
        if veh_link != 0 {
            let vehicle = LEVEL_THINGS_BASE.add(veh_link as usize);
            if *vehicle.add(0x18) == 2 {
                person_goto_in_vehicle(entity, vehicle);
                move_mapwho(entity,
                    DATA_60B28 as i16, DATA_60B2A as i16, DATA_60B2C as i16);
                return;
            }
        }

        // Walk on foot
        person_goto(entity);
        if person_colide(entity) != 0 {
            decide_on_hug_direction(entity, 0x1f4);
        }
        let new_state = affect_person(entity) as u8;
        *entity.add(0x19) = new_state;
        animate_model(entity);
    }
}

// ---------------------------------------------------------------------------
// 0x32300  fn_S_PERSON_MOVE_INTO_VEHICLE
//
// Board a vehicle stored in entity.field_0x2c. Checks that the entity is
// close enough (xy dist < 128, z diff <= 128). Inserts the entity at the end
// of the vehicle's passenger chain and sets state IN_VEHICLE (8) or delegates
// via new_state_person when not yet at the vehicle.
//
// Literal translation of 0x32300–0x32470.
// ---------------------------------------------------------------------------
pub fn fn_s_person_move_into_vehicle(entity: *mut u8) {
    use crate::syndre::data::LEVEL_THINGS_BASE;
    use crate::syndre::funcs_20000::{move_mapwho, getdist};
    unsafe {
        let esi = entity;
        let dx_follow = *(esi.add(0x20) as *const u16);
        *esi.add(0x54) = 0;

        // If following an entity that is already in a vehicle, abort
        if dx_follow != 0 {
            let follow = LEVEL_THINGS_BASE.add(dx_follow as usize);
            if *(follow.add(0x24) as *const u16) != 0 {
                let al = *esi.add(0x19);
                *esi.add(0x58) = al;
                new_state_person(esi);
                return;
            }
        }
        // Already in a vehicle, abort
        if *(esi.add(0x24) as *const u16) != 0 {
            let al = *esi.add(0x19);
            *esi.add(0x58) = al;
            new_state_person(esi);
            return;
        }

        let veh_link = *(esi.add(0x2c) as *const u16) as usize;
        let edi = LEVEL_THINGS_BASE.add(veh_link); // target vehicle

        // Horizontal distance check
        let dy = (*(edi.add(0x6) as *const i16)).wrapping_sub(*(esi.add(0x6) as *const i16));
        let dx = (*(edi.add(0x4) as *const i16)).wrapping_sub(*(esi.add(0x4) as *const i16));
        let dist = getdist(dx, dy);

        // Vertical distance check
        let ez: i32 = *(esi.add(0x8) as *const i16) as i32;
        let vz: i32 = *(edi.add(0x8) as *const i16) as i32;
        let zdiff = (ez - vz) as i32;
        let abs_z = unsafe { ac_abs(zdiff) };

        if dist >= 0x80 || abs_z > 0x80 {
            // Too far — switch to GOTO_VEHICLE state, desired MOVE_INTO_VEHICLE
            *esi.add(0x19) = 5;
            *esi.add(0x58) = 6;
            return;
        }

        // Close enough — check vehicle alive
        if (*edi.add(0x0b) & 0x1) != 0 {
            let al = *esi.add(0x19);
            *esi.add(0x58) = al;
            new_state_person(esi);
            return;
        }

        *(esi.add(0x0c) as *mut u16) = 0;

        let first_pass_ax = *(edi.add(0x1c) as *const u16);
        if first_pass_ax != 0 {
            // Find last passenger in chain
            let mut ax = first_pass_ax;
            let mut ebx = LEVEL_THINGS_BASE.add(ax as usize);
            loop {
                ax = *(ebx.add(0x22) as *const u16);
                if ax == 0 { break; }
                ebx = LEVEL_THINGS_BASE.add(ax as usize);
            }
            // Insert entity after ebx (last passenger)
            let ebx_off = (ebx as usize).wrapping_sub(LEVEL_THINGS_BASE as usize) as u16;
            *(esi.add(0x24) as *mut u16) = ebx_off;
            let esi_off = (esi as usize).wrapping_sub(LEVEL_THINGS_BASE as usize) as u16;
            *(ebx.add(0x22) as *mut u16) = esi_off;
            *(esi.add(0x22) as *mut u16) = 0;
            *esi.add(0x0a) |= 0x8;
            *esi.add(0x19) = 8;
        } else {
            // Vehicle is empty — become first passenger
            let edi_off = (edi as usize).wrapping_sub(LEVEL_THINGS_BASE as usize) as u16;
            *(esi.add(0x24) as *mut u16) = edi_off;
            *(esi.add(0x22) as *mut u16) = 0;
            let esi_off = (esi as usize).wrapping_sub(LEVEL_THINGS_BASE as usize) as u16;
            *(edi.add(0x1c) as *mut u16) = esi_off;
            *(esi.add(0x22) as *mut u16) = 0;

            let dl = *edi.add(0x19);
            if dl == 9 || dl == 0xa {
                *esi.add(0x0a) |= 0x8;
                *esi.add(0x19) = 8;
            } else {
                *esi.add(0x55) = *edi.add(0x28);
                new_state_person(esi);
                // fall through to common passenger setup
                *esi.add(0x0a) |= 0x1;
                *esi.add(0x54) = 0;
                *(esi.add(0x34) as *mut u16) = *(edi.add(0x4) as *const u16);
                *(esi.add(0x36) as *mut u16) = *(edi.add(0x6) as *const u16);
                let ez_val = *(esi.add(0x8) as *const i16) as i32;
                let vy_val = *(edi.add(0x6) as *const i16) as i32;
                let vx_val = *(edi.add(0x4) as *const i16) as i32;
                move_mapwho(esi, vx_val as i16, vy_val as i16, ez_val as i16);
                return;
            }
        }

        // Common passenger finalisation (jump_32434)
        *esi.add(0x0a) |= 0x1;
        *esi.add(0x54) = 0;
        *(esi.add(0x34) as *mut u16) = *(edi.add(0x4) as *const u16);
        *(esi.add(0x36) as *mut u16) = *(edi.add(0x6) as *const u16);
        let ez_val = *(esi.add(0x8) as *const i16) as i32;
        let vy_val = *(edi.add(0x6) as *const i16) as i32;
        let vx_val = *(edi.add(0x4) as *const i16) as i32;
        move_mapwho(esi, vx_val as i16, vy_val as i16, ez_val as i16);
    }
}

// ---------------------------------------------------------------------------
// 0x32480  fn_S_PERSON_MOVE_OUT_OF_VEHICLE
//
// Remove entity from its vehicle's passenger chain, then call move_mapwho and
// new_state_person. If entity has no vehicle link, relinquish via new_state.
//
// Literal translation of 0x32480–0x32591.
// ---------------------------------------------------------------------------
pub fn fn_s_person_move_out_of_vehicle(entity: *mut u8) {
    use crate::syndre::data::LEVEL_THINGS_BASE;
    use crate::syndre::funcs_20000::move_mapwho;
    unsafe {
        let ebx = entity;
        let dx = *(ebx.add(0x24) as *const u16);
        *ebx.add(0x54) = 0;

        if dx == 0 {
            let al = *ebx.add(0x19);
            *ebx.add(0x58) = al;
            new_state_person(ebx);
            return;
        }

        let eax = LEVEL_THINGS_BASE.add(dx as usize); // vehicle
        *(ebx.add(0x0c) as *mut u16) = 0;

        let si = *(ebx.add(0x22) as *const u16); // next passenger link
        let edx_veh = eax; // vehicle ptr

        if *eax.add(0x18) == 2 {
            // Vehicle is type 2 — splice from field_0x1c chain
            if si != 0 {
                let next_pass = LEVEL_THINGS_BASE.add(si as usize);
                let cx = *(ebx.add(0x24) as *const u16);
                *(next_pass.add(0x24) as *mut u16) = cx;
            }
            let ax = *(ebx.add(0x22) as *const u16);
            *(edx_veh.add(0x1c) as *mut u16) = ax;
        } else {
            // Non-type-2 — splice from field_0x22 chain
            let cx = *(ebx.add(0x22) as *const u16);
            if cx != 0 {
                let next_pass = LEVEL_THINGS_BASE.add(cx as usize);
                let cx2 = *(ebx.add(0x24) as *const u16);
                *(next_pass.add(0x24) as *mut u16) = cx2;
            }
            let ax = *(ebx.add(0x22) as *const u16);
            *(edx_veh.add(0x22) as *mut u16) = ax;
        }

        // Restore desired command and exit vehicle
        *ebx.add(0x55) = *ebx.add(0x56);
        let ex = *(ebx.add(0x8) as *const i16) as i32;
        let ey = (*(ebx.add(0x6) as *const i16) as i32).wrapping_add(1);
        let ex2 = (*(ebx.add(0x4) as *const i16) as i32).wrapping_add(1);
        *(ebx.add(0x24) as *mut u16) = 0;
        *(ebx.add(0x22) as *mut u16) = 0;
        let dh = *ebx.add(0x0a) & 0xfe;
        *ebx.add(0x54) = 0;
        *ebx.add(0x0a) = dh;
        move_mapwho(ebx, ex2 as i16, ey as i16, ex as i16);
        new_state_person(ebx);
    }
}

// ---------------------------------------------------------------------------
// 0x32560  fn_S_PERSON_PASSENGER
//
// Keep entity's mapwho position in sync with the vehicle it is riding. If the
// vehicle has stopped (states 9/0xa) and the entity has a follow-target that
// could disembark, trigger a new state. Otherwise move_mapwho to the first
// type-2 vehicle in the chain.
//
// Literal translation of 0x32560–0x32761.
// ---------------------------------------------------------------------------
pub fn fn_s_person_passenger(entity: *mut u8) {
    use crate::syndre::data::{LEVEL_THINGS_BASE, DATA_55358};
    use crate::syndre::funcs_20000::move_mapwho;
    unsafe {
        let esi = entity;
        *esi.add(0x54) = 0;

        let bx_veh = *(esi.add(0x24) as *const u16);
        let ebx = LEVEL_THINGS_BASE.add(bx_veh as usize); // current vehicle
        *(esi.add(0x0c) as *mut u16) = 0;
        let edi = ebx; // save vehicle ptr

        let dx_follow = *(esi.add(0x20) as *const u16);
        if dx_follow != 0 {
            let follow = LEVEL_THINGS_BASE.add(dx_follow as usize);
            if *(follow.add(0x24) as *const u16) == 0 {
                // Follow target not in vehicle — check map tile under entity
                let ey_raw = *(esi.add(0x6) as *const i16) as i32;
                let ex_raw = *(esi.add(0x4) as *const i16) as i32;
                let ez_raw = *(esi.add(0x8) as *const i16) as i32;

                // Compute tile coords (divide by 0x6000 with signed remainder)
                let tile_y = {
                    let r = ey_raw % 0x6000;
                    if r < 0 { (r + 0x6000) } else { r }
                } >> 8;
                let tile_x = ((ex_raw & 0xff00_i32) as i32) >> 8;

                // Look up map: data_55358[tile_y * 128 + tile_x] → type byte
                let tile_idx = (tile_y as usize).wrapping_mul(128)
                    .wrapping_add(tile_x as usize);
                let map_ptr = DATA_55358;
                if !map_ptr.is_null() {
                    let tile_base = *(map_ptr.add(tile_idx * 4) as *const u8);
                    if tile_base != 2 {
                        *esi.add(0x19) = 7;
                        *esi.add(0x0a) &= !0x8;
                        *esi.add(0x58) = 0x1e;
                        return;
                    }
                }
                let _ = ez_raw; // suppress unused warning
            }
        }

        // Check vehicle active and in load/unload states (9/a)
        if *edi.add(0x18) == 2 {
            let veh_state = *edi.add(0x19);
            if veh_state == 9 || veh_state == 0xa {
                // Vehicle stopped — check if we need to reposition
                let stored_x = *(esi.add(0x34) as *const u16);
                let stored_y = *(esi.add(0x36) as *const u16);
                let veh_x = *(ebx.add(0x4) as *const u16);
                let veh_y = *(ebx.add(0x6) as *const u16);
                if stored_x != veh_x || stored_y != veh_y {
                    // Follow chain to find tail entity
                    let mut ax = *(ebx.add(0x20) as *const u16);
                    let mut ecx = edi; // default: vehicle itself
                    if ax != 0 {
                        loop {
                            ecx = LEVEL_THINGS_BASE.add(ax as usize);
                            ax = *(ecx.add(0x20) as *const u16);
                            if ax == 0 { break; }
                        }
                    }

                    // Tail entity's first passenger (field_0x1c) — check animating
                    let first_pass_link = *(ecx.add(0x1c) as *const u16);
                    if first_pass_link != 0 {
                        let first_pass = LEVEL_THINGS_BASE.add(first_pass_link as usize);
                        let dh = *first_pass.add(0x54);
                        if dh == 0 {
                            // Set entity angle based on lead angle
                            let bl = *ecx.add(0x1a); // lead vehicle angle
                            if bl == 0 || bl == 0x80 {
                                *esi.add(0x1a) = 0x40;
                            } else {
                                *esi.add(0x1a) = 0; // dh was 0
                            }
                            *esi.add(0x58) = 7;
                            new_state_person(esi);
                            return;
                        }
                    }
                }
                // fall through to 326a0
            }
        }

        // 326a0: vehicle not in states 9/a — check for exit
        if *edi.add(0x18) == 2 {
            let ch = *edi.add(0x19);
            if ch != 9 && ch != 0xa {
                let veh_cmd = *ebx.add(0x28);
                *esi.add(0x55) = veh_cmd;
                *esi.add(0x0a) &= !0x8;
                let al = *esi.add(0x19);
                *esi.add(0x58) = al;
                new_state_person(esi);
                return;
            }
        }

        // 326d6: Find first type-2 vehicle in entity's vehicle chain
        let mut ax = *(esi.add(0x24) as *const u16);
        let mut ebx_cur = edi;
        if ax != 0 {
            loop {
                ebx_cur = LEVEL_THINGS_BASE.add(ax as usize);
                if *ebx_cur.add(0x18) == 2 { break; }
                ax = *(ebx_cur.add(0x24) as *const u16);
                if ax == 0 { break; }
            }
        }

        let vx = *(ebx_cur.add(0x4) as *const i16) as i32;
        let vy = *(ebx_cur.add(0x6) as *const i16) as i32;
        let vz = *(ebx_cur.add(0x8) as *const i16) as i32;
        move_mapwho(esi, vx as i16, vy as i16, vz as i16);
    }
}

// ---------------------------------------------------------------------------
// 0x32720  fn_S_PERSON_PICKUP_WEAPON
//
// Pick up the weapon stored in entity.field_0x2c if within reach. Animates
// the pickup, clamps weapon ammo to WEAPON_MAX_AMMO, inserts weapon into
// entity's weapon list, and calls new_state_person.
//
// Literal translation of 0x32720–0x32864.
// ---------------------------------------------------------------------------
pub fn fn_s_person_pickup_weapon(entity: *mut u8) {
    use crate::syndre::data::{LEVEL_THINGS_BASE, WEAPON_MAX_AMMO};
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let esi = entity;
        let ebx = LEVEL_THINGS_BASE.add(*(esi.add(0x2c) as *const u16) as usize);
        *esi.add(0x54) = 0;

        // Position check
        let at_x = *(esi.add(0x4) as *const u16) == *(ebx.add(0x4) as *const u16);
        let at_y = *(esi.add(0x6) as *const u16) == *(ebx.add(0x6) as *const u16);
        let ez: i32 = *(esi.add(0x8) as *const i16) as i32;
        let wz: i32 = *(ebx.add(0x8) as *const i16) as i32;
        let abs_z = unsafe { ac_abs(ez.wrapping_sub(wz)) };

        if !at_x || !at_y || abs_z > 0x80 {
            *esi.add(0x19) = 5;
            *esi.add(0x58) = 9;
            return;
        }

        if (*ebx.add(0x0a) & 0x1) != 0 {
            // Weapon busy — just call new_state
            new_state_person(esi);
            return;
        }

        let anim_done = animate_model(esi);
        if anim_done == 0 { return; }

        // Clamp ammo
        let ammo_i16 = *(ebx.add(0x14) as *const i16);
        *(ebx.add(0x1c) as *mut u16) = 0;
        if ammo_i16 >= 0 {
            let wtype = *ebx.add(0x19) as usize;
            if wtype < WEAPON_MAX_AMMO.len() {
                let max_ammo = WEAPON_MAX_AMMO[wtype] as i16;
                if ammo_i16 > max_ammo {
                    *(ebx.add(0x14) as *mut i16) = max_ammo;
                }
            }
        }

        // Insert weapon into entity's weapon list
        let weapon_off = (ebx as usize).wrapping_sub(LEVEL_THINGS_BASE as usize) as u16;
        let ent_off    = (esi as usize).wrapping_sub(LEVEL_THINGS_BASE as usize) as u16;

        let existing = *(esi.add(0x3a) as *const u16);
        if existing != 0 {
            // Walk existing weapon chain to find last
            let mut eax = LEVEL_THINGS_BASE.add(existing as usize);
            loop {
                let next = *(eax.add(0x1c) as *const u16);
                if next == 0 { break; }
                eax = LEVEL_THINGS_BASE.add((next & 0xffff) as usize);
            }
            let last_off = (eax as usize).wrapping_sub(LEVEL_THINGS_BASE as usize) as u16;
            *(ebx.add(0x1e) as *mut u16) = last_off;
            *(eax.add(0x1c) as *mut u16) = weapon_off;
        } else {
            *(esi.add(0x3a) as *mut u16) = weapon_off;
            *(ebx.add(0x1e) as *mut u16) = ent_off;
        }

        *ebx.add(0x0a) |= 0x1;
        *(ebx.add(0x20) as *mut u16) = ent_off;
        *esi.add(0x1a) = 0x20;
        new_state_person(esi);
    }
}

// ---------------------------------------------------------------------------
// 0x32840  fn_S_PERSON_DROP_WEAPON
//
// Drop entity's held weapon, animate, and if animation finished call
// new_state_person with angle 0x20.
//
// Literal translation of 0x32840–0x32888.
// ---------------------------------------------------------------------------
pub fn fn_s_person_drop_weapon(entity: *mut u8) {
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        *entity.add(0x54) = 0;
        drop_weapon(entity);
        let done = animate_model(entity);
        if done != 0 {
            *entity.add(0x1a) = 0x20;
            new_state_person(entity);
        }
    }
}

// ---------------------------------------------------------------------------
// 0x32870  fn_S_PERSON_SELECT_WEAPON
//
// Copy entity.field_0x2c (target weapon offset) into entity.field_0x44
// (selected weapon) then call new_state_person.
//
// Literal translation of 0x32870–0x32900.
// ---------------------------------------------------------------------------
pub fn fn_s_person_select_weapon(entity: *mut u8) {
    unsafe {
        let dx = *(entity.add(0x2c) as *const u16);
        *(entity.add(0x44) as *mut u16) = dx;
        new_state_person(entity);
    }
}

// ---------------------------------------------------------------------------
// 0x32890  fn_S_PERSON_WAIT_FOR_MODEL
//
// Wait until entity's map-tile position matches the target entity's tile
// position (field_0x2c). When aligned, set state 0xd and field_0x2c = 0x14.
// Always calls affect_person.
//
// Literal translation of 0x32890–0x328d4.
// ---------------------------------------------------------------------------
pub fn fn_s_person_wait_for_model(entity: *mut u8) {
    use crate::syndre::data::LEVEL_THINGS_BASE;
    unsafe {
        let ebx = entity;
        let target_link = *(ebx.add(0x2c) as *const u16) as usize;
        let eax = LEVEL_THINGS_BASE.add(target_link);
        *ebx.add(0x54) = 0;

        // Compare tile coords (field_0x4 and field_0x6 >> 8)
        let ex_tile = (*(ebx.add(0x4) as *const i16) as i32) >> 8;
        let tx_tile = (*(eax.add(0x4) as *const i16) as i32) >> 8;
        let ey_tile = (*(ebx.add(0x6) as *const i16) as i32) >> 8;
        let ty_tile = (*(eax.add(0x6) as *const i16) as i32) >> 8;

        if ex_tile == tx_tile && ey_tile == ty_tile {
            *ebx.add(0x19) = 0xd;
            *(ebx.add(0x2c) as *mut u16) = 0x14;
        }

        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x328e0  fn_S_PERSON_WAIT_FOR_TIME
//
// Count down entity.field_0x2c. When it reaches zero (or is > 500), clear
// field_0x44 and field_0x46 and call new_state_person. Always calls
// affect_person and stores result in field_0x19.
//
// Literal translation of 0x328e0–0x32917.
// ---------------------------------------------------------------------------
pub fn fn_s_person_wait_for_time(entity: *mut u8) {
    unsafe {
        let ebx = entity;
        let dx = *(ebx.add(0x2c) as *const u16);
        *ebx.add(0x54) = 0;

        if dx == 0 || dx > 0x1f4 {
            *(ebx.add(0x44) as *mut u16) = 0;
            *ebx.add(0x46) = 0;
            new_state_person(ebx);
        } else {
            *(ebx.add(0x2c) as *mut u16) = dx.wrapping_sub(1);
        }

        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// Shared inner loop for fn_S_PERSON_HUG_RIGHT / HUG_LEFT.
//
// Checks whether entity has reached its tile-level goto target, syncs drug
// stats from the follow target, then calls do_a_hug(entity, dir). On success
// moves entity via goto_angle + move_mapwho, animates, and calls affect_person.
// Called from HUG_RIGHT (dir = -64) and HUG_LEFT (dir = +64).
// ---------------------------------------------------------------------------
#[inline]
fn hug_common(entity: *mut u8, dir: i32) {
    use crate::syndre::data::{LEVEL_THINGS_BASE, DATA_60B28, DATA_60B2A, DATA_60B2C, DATA_5E128};
    use crate::syndre::funcs_20000::{move_mapwho, animate_model, goto_angle};
    unsafe {
        let ebx = entity;

        // Check tile-level arrival (entity.field_0x2e/30 vs DATA_60B28/2A, both >> 8)
        let ex_tile = (*(ebx.add(0x2e) as *const i16) as i32) >> 8;
        let ey_tile = (*(ebx.add(0x30) as *const i16) as i32) >> 8;
        let tx_tile = (DATA_60B28 as i32) >> 8;
        let ty_tile = (DATA_60B2A as i32) >> 8;
        if ex_tile == tx_tile && ey_tile == ty_tile {
            // Arrived at target
            new_state_person(ebx);
            return;
        }

        let follow_link = *(ebx.add(0x20) as *const u16);
        if follow_link != 0 {
            let esi = LEVEL_THINGS_BASE.add(follow_link as usize);

            // If follow target has different goto target → relinquish
            let fx_tile = (*(esi.add(0x2e) as *const i16) as i32) >> 8;
            let fy_tile = (*(esi.add(0x30) as *const i16) as i32) >> 8;
            if fx_tile != ex_tile || fy_tile != ey_tile {
                new_state_person(ebx);
            }

            // Sync drug stats from follow target
            *ebx.add(0x49) = *esi.add(0x49);
            *ebx.add(0x4d) = *esi.add(0x4d);
            *ebx.add(0x51) = *esi.add(0x51);

            let ax = fatal_weapon(esi);
            let chosen = if ax != 0 { choose_best_weapon(ebx, 0) } else { 0 };
            *(ebx.add(0x44) as *mut u16) = chosen;

            if (*esi.add(0x0b) & 0x1) != 0 {
                // Follow target dead — stop following
                *(ebx.add(0x20) as *mut u16) = 0;
                *ebx.add(0x0a) &= !0x8;
                *ebx.add(0x58) = 0x1e;
                new_state_person(ebx);
                return;
            }

            // Compute speed
            let speed_mod = *ebx.add(0x55) as i32;
            let spd = get_person_speed(ebx, speed_mod) as u8;
            *ebx.add(0x54) = spd;
        }

        // Attempt hug movement
        let hug_ok = do_a_hug(ebx, dir);
        if hug_ok == 0 {
            // Can't hug / arrived — relinquish
            new_state_person(ebx);
            return;
        }

        // Move in hug direction
        let speed_mod = *ebx.add(0x55) as i32;
        let spd = get_person_speed(ebx, speed_mod) as u8;
        *ebx.add(0x54) = spd;

        let angle = *ebx.add(0x1a) as u32 as u16;
        let speed = *ebx.add(0x54) as u32 as u16;
        goto_angle(speed as i16, angle as u8);

        let z_arg = (DATA_60B2C as i16).wrapping_add(DATA_5E128);
        move_mapwho(ebx, DATA_60B28 as i16, DATA_60B2A as i16, z_arg);
        animate_model(ebx);
        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x32930  fn_S_PERSON_HUG_RIGHT
//
// Wall-hug state that turns clockwise (right, dir = -64) when blocked.
// Delegates to hug_common after the per-entity setup.
//
// Literal translation of 0x32930–0x32a97.
// ---------------------------------------------------------------------------
pub fn fn_s_person_hug_right(entity: *mut u8) {
    unsafe { hug_common(entity, -64); }
}

// ---------------------------------------------------------------------------
// 0x32aa0  fn_S_PERSON_HUG_LEFT
//
// Wall-hug state that turns counter-clockwise (left, dir = +64) when blocked.
//
// Literal translation of 0x32aa0–0x32c0e.
// ---------------------------------------------------------------------------
pub fn fn_s_person_hug_left(entity: *mut u8) {
    unsafe { hug_common(entity, 64); }
}

// ---------------------------------------------------------------------------
// 0x32c10  fn_S_PERSON_ON_FIRE
//
// Burning entity state. Decrements the speed-ref counter (field_0x42). When
// it goes negative: if ammo (field_0x14) is negative → state 0x19 (DYING),
// else clear state bits and call new_state_person. While counter >= 0: add
// random ±7 to angle, clamp adrenalin += 6 to 0xff, compute speed, call
// goto_angle + person_colide + animate_model.
//
// Literal translation of 0x32c10–0x32c83.
// ---------------------------------------------------------------------------
pub fn fn_s_person_on_fire(entity: *mut u8) {
    use crate::syndre::funcs_20000::{animate_model, goto_angle};
    unsafe {
        let ebx = entity;
        *ebx.add(0x0b) |= 0x2; // set "on fire" flag

        let ax = *(ebx.add(0x42) as *const i16);
        let new_ax = ax.wrapping_sub(1);
        *(ebx.add(0x42) as *mut i16) = new_ax;

        if ax < 0 {
            // Counter expired
            if *(ebx.add(0x14) as *const i16) < 0 {
                *ebx.add(0x19) = 0x19; // DYING state
                return;
            }
            // Clear on-fire bits and relinquish
            let mut cx = *(ebx.add(0x0a) as *const u16);
            cx &= 0xfdf7; // clear bits 3 and 9
            *(ebx.add(0x0a) as *mut u16) = cx;
            new_state_person(ebx);
            return;
        }

        // Still burning — randomise angle
        let rand_val = random(0xf) as i8; // 0..15
        let jitter = rand_val.wrapping_sub(7) as i8; // -7..8
        let dl = (*ebx.add(0x1a) as i8).wrapping_add(jitter) as u8;
        *ebx.add(0x1a) = dl;

        // Boost adrenalin
        let adrn = *ebx.add(0x49) as i32;
        let clamped = (adrn + 6).min(0xff) as u8;
        *ebx.add(0x49) = clamped;

        // Move in fire direction
        let speed_mod = *ebx.add(0x55) as i32;
        let spd = get_person_speed(ebx, speed_mod) as u8;
        *ebx.add(0x54) = spd;

        let angle = *ebx.add(0x1a) as u32 as u16;
        let speed = *ebx.add(0x54) as u32 as u16;
        goto_angle(speed as i16, angle as u8);
        person_colide(ebx);
        animate_model(ebx);
    }
}

// ---------------------------------------------------------------------------
// Shared helper: compute armor-resistance index from entity.field_0x1d/0x3c.
// Returns 0-3 (or 4 if immune via bit 0x10 of field_0x1d with dx=0 already).
// ---------------------------------------------------------------------------
#[inline]
unsafe fn armor_resistance(entity: *const u8) -> u32 {
    let bl = *entity.add(0x1d);
    if (bl & 0x10) != 0 {
        return 0; // immune — treat as level 0 (max damage)
    }
    let dx = (*(entity.add(0x3c) as *const u16) & 0x60) as u32;
    dx >> 5
}

// ---------------------------------------------------------------------------
// 0x32dc0  fn_S_PERSON_FLY_BACK
//
// Knock-back movement. Sets "hurt" flag on field_0xb bit 1, then calls
// random(speed) to compute fly speed, calls goto_angle + person_colide.
// Decrements ammo (field_0x14) when DATA_5E128 < -64 (z-fall case).
// After animation completes: if ammo < 0 → DEAD_ANIM state (0x18), else
// flip angle 180°, clear on-fire/vehicle bits, set field_0x5b, new_state.
// Always calls affect_person.
//
// Literal translation of 0x32dc0–0x32e87.
// ---------------------------------------------------------------------------
pub fn fn_s_person_fly_back(entity: *mut u8) {
    use crate::syndre::data::DATA_5E128;
    use crate::syndre::funcs_20000::{move_mapwho, animate_model, goto_angle};
    unsafe {
        let ebx = entity;
        *ebx.add(0x0b) |= 0x2;

        let angle = *ebx.add(0x1a);
        let speed = *ebx.add(0x54) as i32;

        // Compute randomised fly speed: speed + (random(speed)+2)/2
        let r = random(speed) as i32;
        let fly_speed = (speed + (r.wrapping_add(2).unsigned_abs() as i32 >> 1)) as u16;
        goto_angle(fly_speed as i16, angle);

        // Update speed: (speed+2)/2
        let new_speed = ((speed.wrapping_add(2)).unsigned_abs() as i32 >> 1) as u8;
        *ebx.add(0x54) = new_speed;
        person_colide(ebx);

        // Fall damage
        if DATA_5E128 < -64 {
            let ammo = *(ebx.add(0x14) as *const i16);
            *(ebx.add(0x14) as *mut i16) = ammo.wrapping_sub(1);
        }

        let done = animate_model(ebx);
        if done != 0 {
            if *(ebx.add(0x14) as *const i16) < 0 {
                *ebx.add(0x19) = 0x18; // DEAD animation state
            } else {
                // Flip direction and enter normal-walk recovery
                let dl = (*ebx.add(0x1a)).wrapping_add(0x80);
                *ebx.add(0x1a) = dl;
                let si = *(ebx.add(0x0a) as *const u16) & 0xfdf7;
                *(ebx.add(0x0a) as *mut u16) = si;
                *ebx.add(0x5b) = dl;
                *ebx.add(0x19) = 0x16;
                new_state_person(ebx);
            }
        }

        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;

        // Suppress unused import warning
        let _ = move_mapwho as usize;
    }
}

// ---------------------------------------------------------------------------
// 0x32c10-area  hit-by helper: decode armor resistance and apply ammo damage.
// Used by HIT_BY_EXPLOSION, HIT_BY_VEHICLE, HIT_BY_FIRE.
//
// resist: 0 = no armor (max damage), 1 = light armor, 2 = medium, 3 = heavy.
// Damage amounts differ per caller; caller sets them in field_0x14 then calls
// fn_S_PERSON_ON_FIRE or fn_S_PERSON_FLY_BACK.
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// 0x32ce0  fn_S_PERSON_HIT_BY_FIRE
//
// Dispatch on armor resistance; decrement ammo by 0/8/4/2 per tick or set
// ammo=-1 (fatal). When ammo goes negative: timer=40, state=ON_FIRE and call
// fn_S_PERSON_ON_FIRE. When ammo >= 0 near miss: timer=2.
//
// Literal translation of 0x32ce0–0x32dbe.
// ---------------------------------------------------------------------------
pub fn fn_s_person_hit_by_fire(entity: *mut u8) {
    unsafe {
        let eax = entity;
        let resist = armor_resistance(eax);
        let ammo_decrement: i16 = match resist {
            0 => {
                // Instant burn: set ammo=-1, long timer, ON_FIRE
                *(eax.add(0x14) as *mut i16) = -1_i16;
                *(eax.add(0x42) as *mut i16) = 40;
                *eax.add(0x19) = 0x17;
                fn_s_person_on_fire(eax);
                return;
            }
            1 => 8,
            2 => 4,
            3 => 2,
            _ => {
                // dx > 3: fire-resistant; just tick ON_FIRE briefly
                *eax.add(0x19) = 0x17;
                fn_s_person_on_fire(eax);
                return;
            }
        };
        let ammo = (*(eax.add(0x14) as *const i16)).wrapping_sub(ammo_decrement);
        *(eax.add(0x14) as *mut i16) = ammo;
        if ammo < 0 {
            *(eax.add(0x42) as *mut i16) = 40;
            *eax.add(0x19) = 0x17;
            fn_s_person_on_fire(eax);
        } else {
            *(eax.add(0x42) as *mut i16) = 2;
            *eax.add(0x19) = 0x17;
            fn_s_person_on_fire(eax);
        }
    }
}

// ---------------------------------------------------------------------------
// 0x32ea0  fn_S_PERSON_HIT_BY_EXPLOSION
//
// Set state=FLY_BACK, copy hug-angle field_0x5b → field_0x1a, speed=0x8c.
// Damage based on armor resistance: 0→ammo=-1, 2→ammo-=8, 3→ammo-=4.
// Calls fn_S_PERSON_FLY_BACK.
//
// Literal translation of 0x32ea0–0x32f32.
// ---------------------------------------------------------------------------
pub fn fn_s_person_hit_by_explosion(entity: *mut u8) {
    unsafe {
        let eax = entity;
        *eax.add(0x19) = 0x16;
        let bl = *eax.add(0x5b);
        *eax.add(0x54) = 0x8c;
        *eax.add(0x1a) = bl;
        let resist = armor_resistance(eax);
        match resist {
            0 => { *(eax.add(0x14) as *mut i16) = -1_i16; }
            2 => { let a = (*(eax.add(0x14) as *const i16)).wrapping_sub(8); *(eax.add(0x14) as *mut i16) = a; }
            3 => { let a = (*(eax.add(0x14) as *const i16)).wrapping_sub(4); *(eax.add(0x14) as *mut i16) = a; }
            _ => {}
        }
        fn_s_person_fly_back(eax);
    }
}

// ---------------------------------------------------------------------------
// 0x32f00  fn_S_PERSON_HIT_BY_BULLET
//
// Set flags if already dead (field_0x14 < 0), speed=0x54, compute fly
// angle from attacker position (entity.field_0x16 → attacker, get_angle),
// state=0x16, then fn_S_PERSON_FLY_BACK.
//
// Literal translation of 0x32f00–0x32f66.
// ---------------------------------------------------------------------------
pub fn fn_s_person_hit_by_bullet(entity: *mut u8) {
    use crate::syndre::data::LEVEL_THINGS_BASE;
    unsafe {
        let ebx = entity;
        if *(ebx.add(0x14) as *const i16) < 0 {
            let ax = *(ebx.add(0x0a) as *const u16) | 0x108;
            *(ebx.add(0x0a) as *mut u16) = ax;
        }
        *ebx.add(0x46) = 0;
        *ebx.add(0x54) = 0x54;

        let atk_link = *(ebx.add(0x16) as *const u16);
        let atk = LEVEL_THINGS_BASE.add(atk_link as usize);
        let dy = (*(ebx.add(0x6) as *const i16)).wrapping_sub(*(atk.add(0x6) as *const i16));
        let dx = (*(ebx.add(0x4) as *const i16)).wrapping_sub(*(atk.add(0x4) as *const i16));

        let angle = get_angle(dx as i32, dy as i32);
        *ebx.add(0x1a) = angle;
        *ebx.add(0x19) = 0x16;
        fn_s_person_fly_back(ebx);
    }
}

// ---------------------------------------------------------------------------
// 0x32f60  fn_S_PERSON_HIT_BY_VEHICLE
//
// Same pattern as HIT_BY_EXPLOSION but speed=0xa0. Damage: 0→-1, 2→-8, 3→-4.
//
// Literal translation of 0x32f60–0x32fc0.
// ---------------------------------------------------------------------------
pub fn fn_s_person_hit_by_vehicle(entity: *mut u8) {
    unsafe {
        let eax = entity;
        *eax.add(0x19) = 0x16;
        let bl = *eax.add(0x5b);
        *eax.add(0x54) = 0xa0;
        *eax.add(0x1a) = bl;
        let resist = armor_resistance(eax);
        match resist {
            0 => { *(eax.add(0x14) as *mut i16) = -1_i16; }
            2 => { let a = (*(eax.add(0x14) as *const i16)).wrapping_sub(8); *(eax.add(0x14) as *mut i16) = a; }
            3 => { let a = (*(eax.add(0x14) as *const i16)).wrapping_sub(4); *(eax.add(0x14) as *mut i16) = a; }
            _ => {}
        }
        fn_s_person_fly_back(eax);
    }
}

// ---------------------------------------------------------------------------
// 0x32fc0  fn_S_PERSON_HIT_BY_LASER
//
// Animate first; if animation completes, apply ammo damage based on
// resistance (0→-1, 2→-8, 3→-4). If ammo < 0: state=DYING(0x19), set
// dead flags in field_0xa, return. Else clear on-fire/vehicle bits,
// new_state_person.
//
// Literal translation of 0x32fc0–0x3306d.
// ---------------------------------------------------------------------------
pub fn fn_s_person_hit_by_laser(entity: *mut u8) {
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let ebx = entity;
        let done = animate_model(ebx);
        if done != 0 {
            let resist = armor_resistance(ebx);
            let ammo_decrement: i16 = match resist {
                0 => {
                    *(ebx.add(0x14) as *mut i16) = -1_i16;
                    -1 // flag
                }
                2 => 8,
                3 => 4,
                _ => 0,
            };
            if ammo_decrement > 0 {
                let a = (*(ebx.add(0x14) as *const i16)).wrapping_sub(ammo_decrement);
                *(ebx.add(0x14) as *mut i16) = a;
            }

            if *(ebx.add(0x14) as *const i16) < 0 {
                *ebx.add(0x19) = 0x19; // DYING
                let ax = *(ebx.add(0x0a) as *const u16) | 0x109;
                *(ebx.add(0x0a) as *mut u16) = ax as u16;
                return;
            }
            let di = *(ebx.add(0x0a) as *const u16) & 0xfdf7;
            *(ebx.add(0x0a) as *mut u16) = di;
            new_state_person(ebx);
        }
    }
}

// ---------------------------------------------------------------------------
// 0x33050  fn_S_PERSON_DYING
//
// Mark entity as blocking (field_0xa |= 0x8, field_0x54 = 0), drop all
// weapons, animate. When animation finishes: set state 0x1a (DEAD), then
// call who_shot_me.
//
// Literal translation of 0x33050–0x33087.
// ---------------------------------------------------------------------------
pub fn fn_s_person_dying(entity: *mut u8) {
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let ebx = entity;
        let ah = *ebx.add(0x0a) | 0x8;
        *ebx.add(0x54) = 0;
        *ebx.add(0x0a) = ah;
        drop_all_weapons(ebx);
        let done = animate_model(ebx);
        if done != 0 {
            *ebx.add(0x19) = 0x1a;
            who_shot_me(ebx);
        }
    }
}

// ---------------------------------------------------------------------------
// 0x33090  fn_S_PERSON_DYING_ON_FIRE
//
// Set field_0xa |= 0x8, kill all weapons, animate. When done: state=0x1b,
// set fire timer field_0x42=100, call who_shot_me.
//
// Literal translation of 0x33090–0x330ca.
// ---------------------------------------------------------------------------
pub fn fn_s_person_dying_on_fire(entity: *mut u8) {
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let ebx = entity;
        *ebx.add(0x0a) |= 0x8;
        kill_all_weapons(ebx);
        let done = animate_model(ebx);
        if done != 0 {
            *ebx.add(0x19) = 0x1b;
            *(ebx.add(0x42) as *mut i16) = 0x64;
            who_shot_me(ebx);
        }
    }
}

// ---------------------------------------------------------------------------
// 0x330d0  fn_S_PERSON_DEAD
//
// Set dead flags, zero animation speed, call animate_model (keep playing
// death animation).
//
// Literal translation of 0x330d0–0x330ee.
// ---------------------------------------------------------------------------
pub fn fn_s_person_dead(entity: *mut u8) {
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let eax = entity;
        let dx = *(eax.add(0x0a) as *const u16) | 0x108;
        *eax.add(0x54) = 0;
        *(eax.add(0x0a) as *mut u16) = dx as u16;
        animate_model(eax);
    }
}

// ---------------------------------------------------------------------------
// 0x330f0  fn_S_PERSON_DEAD_ON_FIRE
//
// Zero animation speed, count down fire timer (field_0x42). When timer
// reaches 0: state = 0x23 (ASH). Always set dead flags and animate.
//
// Literal translation of 0x330f0–0x33117.
// ---------------------------------------------------------------------------
pub fn fn_s_person_dead_on_fire(entity: *mut u8) {
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let eax = entity;
        *eax.add(0x54) = 0;
        let dx = (*(eax.add(0x42) as *const i16)).wrapping_sub(1);
        *(eax.add(0x42) as *mut i16) = dx;
        if dx == 0 {
            *eax.add(0x19) = 0x23; // ASH state
        }
        let cx = *(eax.add(0x0a) as *const u16) | 0x108;
        *(eax.add(0x0a) as *mut u16) = cx as u16;
        animate_model(eax);
    }
}

// ---------------------------------------------------------------------------
// 0x33120  fn_S_PERSON_GUARD_AREA
//
// Intelligence-based enemy detection + weapon use. Gets weapon range from
// DATA_5A6C2[weapon_type], scales by perception via get_person_perception,
// calls agent_check_arc_for_enemy. If an enemy is found, fires via
// person_use_weapon. Decrements guard timer (field_0x42), triggers
// new_state_person when timer reaches 0, then affect_person.
//
// Literal translation of 0x33120–0x33852.
// ---------------------------------------------------------------------------
pub fn fn_s_person_guard_area(entity: *mut u8) {
    use crate::syndre::data::{LEVEL_THINGS_BASE, DATA_5A6C2, DATA_5E12E};
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let ebx = entity;
        *ebx.add(0x54) = 0;

        let edi = get_person_intelligence(ebx, 100) as i32;
        let dx = *(ebx.add(0x44) as *const u16);
        *ebx.add(0x46) = 0;

        if dx != 0 {
            let esi = LEVEL_THINGS_BASE.add(dx as usize); // held weapon

            if *(esi.add(0x14) as *const i16) < 0 {
                // Weapon out of ammo — pick a new one
                let ax = choose_best_weapon(ebx, 0);
                let cur = *(ebx.add(0x44) as *const u16);
                if ax == cur {
                    new_state_person(ebx); // no usable weapon
                } else {
                    *(ebx.add(0x44) as *mut u16) = ax;
                }
            }

            // Compute weapon detection range: DATA_5A6C2[weapon_type] abs >> 8
            let wtype = *esi.add(0x19) as usize;
            let raw_range = if wtype < DATA_5A6C2.len() { DATA_5A6C2[wtype] as i32 } else { 0 };
            // Compute abs(raw_range) >> 8 faithfully (signed sar, sub)
            let range_sig = raw_range as i32;
            let range_abs_shr8 = (range_sig.unsigned_abs() as i32) >> 8;
            let perception_range = get_person_perception(ebx, range_abs_shr8);
            DATA_5E12E = (perception_range << 8) as i16;

            let enemy = agent_check_arc_for_enemy(ebx, DATA_5E12E as i32, edi);
            if !enemy.is_null() {
                let tz = (*(enemy.add(0x8) as *const i16) as i32).wrapping_add(0x80);
                let ty = *(enemy.add(0x6) as *const i16) as i32;
                let tx = *(enemy.add(0x4) as *const i16) as i32;
                person_use_weapon(ebx, tx, ty, tz);
            }
        }

        animate_model(ebx);
        let cx = *(ebx.add(0x42) as *const i16);
        if cx == 0 {
            new_state_person(ebx);
        }
        let si = cx.wrapping_sub(1);
        *(ebx.add(0x42) as *mut i16) = si;
        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x33230  fn_S_PERSON_WANDER
//
// Random walk: compute displacement from sin/cos tables at speed 0x10,
// update DATA_60B28/2A. On collision, call quick_decide_on_hug_direction.
// Check tile under entity (via data_55358 map pointer): if tile 0x80
// (water barrier going east) or 0x81 (going south), redirect angle and
// set state 0x29 (WANDER_WAIT) with timer 100. Add random jitter ±1 to
// angle, set "civilian" flag bit, animate and affect_person.
//
// Literal translation of 0x33230–0x33362.
// ---------------------------------------------------------------------------
pub fn fn_s_person_wander(entity: *mut u8) {
    use crate::syndre::data::{DATA_55358, DATA_5AB60, DATA_5AD60, DATA_60B28, DATA_60B2A};
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let ebx = entity;
        *ebx.add(0x54) = 0x10; // fixed walk speed

        let cl = *ebx.add(0x1a) as usize; // angle index
        let al = *ebx.add(0x54) as i32;   // speed = 16

        // X displacement: sin * speed >> 8
        let sin_val = *(DATA_5AB60.as_ptr().add(cl * 2) as *const i16) as i32;
        let dx_disp = (sin_val.wrapping_mul(al)) >> 8;
        DATA_60B28 = DATA_60B28.wrapping_add(dx_disp as i16);

        // Y displacement: cos * speed >> 8
        let cos_val = *(DATA_5AD60.as_ptr().add(cl * 2) as *const i16) as i32;
        let dy_disp = (cos_val.wrapping_mul(al)) >> 8;
        DATA_60B2A = DATA_60B2A.wrapping_add(dy_disp as i16);

        if person_colide(ebx) != 0 {
            quick_decide_on_hug_direction(ebx, 0xa);
        }

        // Compute tile x/y
        let ey_raw = *(ebx.add(0x6) as *const i16) as i32;
        let ex_raw = *(ebx.add(0x4) as *const i16) as i32;
        let ez_raw = *(ebx.add(0x8) as *const i16) as i32;

        // tile_y = signed_remainder(ey, 0x6000) >> 8
        let rem_y = ey_raw % 0x6000;
        let tile_y = if rem_y < 0 {
            let r = rem_y + 0x6000;
            ((r as i32) - ((r as i32 >> 0x1f) << 8)) >> 8
        } else {
            ((rem_y as i32) - ((rem_y as i32 >> 0x1f) << 8)) >> 8
        };

        // tile_x = ((ex_raw & 0xff00) as i32 - sign_ext*256) >> 8
        let ex_masked = (ex_raw & 0xff00_i32) as i32;
        let tile_x = (ex_masked - (ex_masked >> 31).wrapping_mul(256)) >> 8;

        // Map cell pointer: data_55358 + (tile_y * 128 + tile_x) * 4 + z_level * 4
        let map_ptr = DATA_55358;
        if !map_ptr.is_null() {
            let cell_idx = ((tile_y as usize).wrapping_mul(128))
                .wrapping_add(tile_x as usize);
            let cell_base = *(map_ptr.add(cell_idx * 4) as *const *mut u8);
            if !cell_base.is_null() {
                // z index: (entity.z - 1) >> 7
                let z_idx = ((ez_raw.wrapping_sub(1)).unsigned_abs() as i32 >> 7) as usize;
                let tile_byte = *cell_base.add(z_idx * 4); // each z level = 4 bytes apart? From asm: (%ecx) = first byte

                if tile_byte == 0x80 {
                    // Water/barrier heading east: if angle in [0x80, 0x100], redirect to 0xc0
                    let dh = *ebx.add(0x1a) as u32;
                    if dh >= 0x80 && dh <= 0xff {
                        *ebx.add(0x1a) = 0xc0;
                        *ebx.add(0x19) = 0x29;
                        *(ebx.add(0x42) as *mut i16) = 0x64;
                    }
                } else if tile_byte == 0x81 {
                    // Water/barrier heading north: if angle in [0x40, 0xc0], redirect to 0x80
                    let ah = *ebx.add(0x1a);
                    if ah >= 0x40 && ah <= 0xc0 {
                        *ebx.add(0x1a) = 0x80;
                        *ebx.add(0x19) = 0x29;
                        *(ebx.add(0x42) as *mut i16) = 0x64;
                    }
                }
            }
        }

        // Random angle jitter: ±1 (random(3) - 1)
        let jitter = (random(3) - 1) as i8;
        let cl_byte = (*ebx.add(0x1a) as i8).wrapping_add(jitter) as u8;
        *ebx.add(0x1a) = cl_byte;

        // Set "civilian visible" flag
        *ebx.add(0x0b) |= 0x10;

        animate_model(ebx);
        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x33370  fn_S_PERSON_PERSUADED
//
// Handles a persuaded (mind-controlled) person following their leader.
// Copies leader's drug stats, tracks their goto target, uses bump_person to
// avoid collisions, and steers angle ±0x10 per tick toward the leader.
// ---------------------------------------------------------------------------
pub fn fn_s_person_persuaded(entity: *mut u8) {
    use crate::syndre::funcs_20000::{animate_model, goto_angle};
    unsafe {
        let ebx = entity;
        let dx = *(ebx.add(0x20) as *const u16);
        if dx == 0 {
            return;
        }
        let mut esi = LEVEL_THINGS_BASE.add(dx as usize);

        // If leader is deleted (field_0xb bit0 set): disengage
        if (*esi.add(0x0b) & 0x1) != 0 {
            *(ebx.add(0x20) as *mut u16) = 0;
            let dl = *ebx.add(0xa) & 0xf7;
            *ebx.add(0xa) = dl;
            *ebx.add(0x58) = 0x1e; // desired = FOLLOW
            new_state_person(ebx);
            return;
        }

        // Walk leader's vehicle chain looking for type-2 vehicle
        let mut vdx = *(esi.add(0x24) as *const u16);
        if vdx != 0 {
            loop {
                if vdx == 0 {
                    break;
                }
                let veax = LEVEL_THINGS_BASE.add(vdx as usize);
                if *veax.add(0x18) == 2 {
                    *ebx.add(0x19) = 0x5;  // GOTO_VEHICLE
                    *ebx.add(0x58) = 0x6;  // desired = MOVE_INTO_VEHICLE
                    *(ebx.add(0x2c) as *mut u16) = vdx;
                    return;
                }
                esi = veax;
                vdx = *(veax.add(0x24) as *const u16);
            }
        }

        // Restore esi to the original follow target (vehicle walk may have changed it)
        esi = LEVEL_THINGS_BASE.add(dx as usize);

        let cx = *(ebx.add(0x3a) as *const u16);

        if cx == 0 {
            // Search mapwho tile for free weapons to pick up
            let ey = *(ebx.add(0x6) as *const i16) as i32;
            let ex = *(ebx.add(0x4) as *const i16) as i32;
            let tile_y = ((ey >> 8) & 0x7f) * 128;
            let tile_x = (ex >> 8) & 0x7f;
            let mapwho_idx = (tile_y | tile_x) as usize;

            if !LEVEL_MAPWHO.is_null() {
                let mut ax = *(LEVEL_MAPWHO.add(mapwho_idx * 2) as *const u16) as usize;
                loop {
                    if ax == 0 {
                        break;
                    }
                    let thing = LEVEL_THINGS_BASE.add(ax);
                    if *thing.add(0x18) == 4 {  // type == WEAPON
                        let ammo = *(thing.add(0x14) as *const i16);
                        let owned = *(thing.add(0x1e) as *const u16);
                        let wz = *(thing.add(0x8) as *const i16);
                        let ez = *(ebx.add(0x8) as *const i16);
                        if ammo > 0 && owned == 0 && wz == ez {
                            *ebx.add(0x19) = 0x5;  // GOTO_STRUCT
                            let offset = (thing as usize)
                                .wrapping_sub(LEVEL_THINGS_BASE as usize) as u16;
                            *ebx.add(0x58) = 0x9;  // desired = PICKUP
                            *(ebx.add(0x2c) as *mut u16) = offset;
                            break;
                        }
                    }
                    ax = *(thing.add(0x0) as *const u16) as usize;
                }
            }
        } else {
            // Check if currently held weapon has negative ammo: drop it
            let weap = LEVEL_THINGS_BASE.add(cx as usize);
            let ammo = *(weap.add(0x14) as *const i16);
            if ammo < 0 {
                *(ebx.add(0x44) as *mut u16) = cx;
                drop_weapon(ebx);
            }
        }

        // jump_33484: if leader is itself following someone else, chain to that target
        {
            let ax2 = *(esi.add(0x20) as *const u16);
            if ax2 != 0 {
                *(ebx.add(0x20) as *mut u16) = ax2;
                return;
            }
        }

        // Compute perception range for bump sphere (clamped to >= 16)
        let perc_raw = get_person_perception(esi, 0xc0);
        let mut edi = (perc_raw as i32).wrapping_add(0x40);
        if edi < 0x10 {
            edi = 0x10;
        }

        // Copy leader's drug enhancement stats
        *ebx.add(0x49) = *esi.add(0x49);
        *ebx.add(0x4d) = *esi.add(0x4d);
        *ebx.add(0x51) = *esi.add(0x51);

        // Select best weapon; if leader has a fatal weapon, choose for entity
        let fw = fatal_weapon(esi);
        let chosen: u16 = if fw != 0 {
            choose_best_weapon(ebx, 0)
        } else {
            0
        };
        *(ebx.add(0x44) as *mut u16) = chosen;

        // Set walk speed
        let speed_modifier = *ebx.add(0x55) as i32;
        let spd = get_person_speed(ebx, speed_modifier);
        *ebx.add(0x54) = spd as u8;

        // Copy leader's goto target coords
        *(ebx.add(0x2e) as *mut i16) = *(esi.add(0x2e) as *const i16);
        *(ebx.add(0x30) as *mut i16) = *(esi.add(0x30) as *const i16);

        // Compute bump target using sin/cos tables + DATA_60B2x accumulators
        let angle_idx = *ebx.add(0x1a) as usize;
        let spd_val = *ebx.add(0x54) as i32;
        let sin_val = *(DATA_5AB60.as_ptr().add(angle_idx * 2) as *const i16) as i32;
        let cos_val = *(DATA_5AD60.as_ptr().add(angle_idx * 2) as *const i16) as i32;
        let tx = (DATA_60B28 as i32).wrapping_add((sin_val.wrapping_mul(spd_val)) >> 8);
        let ty = (DATA_60B2A as i32).wrapping_add((cos_val.wrapping_mul(spd_val)) >> 8);
        let tz = DATA_60B2C as i32;

        let bump_result = bump_person(ebx, tx, ty, tz, edi, edi, 0x80);

        // Compute signed diff between desired angle and entity's current angle
        let al: u8 = if !bump_result.is_null() {
            // Steer toward bump collision entity
            let col_y = *(bump_result.add(0x6) as *const i16) as i32;
            let ent_y2 = *(ebx.add(0x6) as *const i16) as i32;
            let col_x = *(bump_result.add(0x4) as *const i16) as i32;
            let ent_x2 = *(ebx.add(0x4) as *const i16) as i32;
            let computed = get_angle(col_x.wrapping_sub(ent_x2), col_y.wrapping_sub(ent_y2));
            computed.wrapping_sub(*ebx.add(0x1a))
        } else {
            // Steer toward goto target
            let tgt_y2 = *(ebx.add(0x30) as *const i16) as i32;
            let ent_y2 = *(ebx.add(0x6) as *const i16) as i32;
            let tgt_x2 = *(ebx.add(0x2e) as *const i16) as i32;
            let ent_x2 = *(ebx.add(0x4) as *const i16) as i32;
            let computed = get_angle(tgt_x2.wrapping_sub(ent_x2), tgt_y2.wrapping_sub(ent_y2));
            (*ebx.add(0x1a)).wrapping_sub(computed)
        };

        // Rotate entity angle toward target by at most 0x10 per tick
        let diff = al as i8;
        if diff < 0 {
            if diff >= -16 {
                *ebx.add(0x1a) = (*ebx.add(0x1a)).wrapping_sub(al);
            } else {
                *ebx.add(0x1a) = (*ebx.add(0x1a)).wrapping_add(0x10);
            }
        } else if diff > 0 {
            if diff <= 16 {
                *ebx.add(0x1a) = (*ebx.add(0x1a)).wrapping_add(al);
            } else {
                *ebx.add(0x1a) = (*ebx.add(0x1a)).wrapping_sub(0x10);
            }
        }

        let cur_angle = *ebx.add(0x1a);
        let cur_speed = *ebx.add(0x54);
        goto_angle(cur_speed as i16, cur_angle);

        if person_colide(ebx) != 0 {
            let dy_g = (*(ebx.add(0x30) as *const i16) as i32)
                .wrapping_sub(*(ebx.add(0x6) as *const i16) as i32);
            let dx_g = (*(ebx.add(0x2e) as *const i16) as i32)
                .wrapping_sub(*(ebx.add(0x4) as *const i16) as i32);
            let target_angle = get_angle(dx_g, dy_g);
            let adl = target_angle.wrapping_sub(*ebx.add(0x1a)) as i8 as i32;
            let hug_speed: u32 = if adl > -16 && (adl as u8) < 0x10 { 0x1f4 } else { 0x14 };
            decide_on_hug_direction(ebx, hug_speed);
        }

        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
        animate_model(ebx);
    }
}

// ---------------------------------------------------------------------------
// 0x33660  fn_S_PERSON_RUNAWAY
//
// Flee from entity at field_0x2a. Computes opposite angle using arctan,
// moves at speed 0x28. Stops when distance > perception range.
// ---------------------------------------------------------------------------
pub fn fn_s_person_runaway(entity: *mut u8) {
    use crate::syndre::funcs_20000::{animate_model, goto_angle};
    use crate::syndre::funcs_10000::getrdist;
    unsafe {
        let ebx = entity;
        let si = *(ebx.add(0x2a) as *const u16);
        let esi = LEVEL_THINGS_BASE.add(si as usize);

        if *esi.add(0x18) == 0 {
            new_state_person(ebx);
            return;
        }

        let threat_y = *(esi.add(0x6) as *const i16) as i32;
        let ent_y   = *(ebx.add(0x6) as *const i16) as i32;
        let threat_x = *(esi.add(0x4) as *const i16) as i32;
        let ent_x   = *(ebx.add(0x4) as *const i16) as i32;
        let dy = threat_y.wrapping_sub(ent_y);
        let dx = threat_x.wrapping_sub(ent_x);
        // arctan(dx, dy) → +0x80 gives the opposite (flee) direction
        let angle = crate::syndre::funcs_40000::arctan(dx as i16, dy as i16)
            .wrapping_add(0x80) as u8;
        *ebx.add(0x1a) = angle;
        *ebx.add(0x54) = 0x28;

        goto_angle(0x28i16, angle);

        if person_colide(ebx) != 0 {
            quick_decide_on_hug_direction(ebx, 0x32);
        }

        // Check escape distance vs perception range
        let dy2 = (*(esi.add(0x6) as *const i16) as i32)
            .wrapping_sub(*(ebx.add(0x6) as *const i16) as i32);
        let dx2 = (*(esi.add(0x4) as *const i16) as i32)
            .wrapping_sub(*(ebx.add(0x4) as *const i16) as i32);
        let si_dist = getrdist(dx2 as i16, dy2 as i16) as u32;
        let perception = get_person_perception(ebx, 0xa00) as u32;
        if si_dist > perception {
            new_state_person(ebx);
        }

        animate_model(ebx);
        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x33740  fn_S_PERSON_GIVE_WARNING
//
// Warning state: stand still, face the threat, count down timer. If timer
// hits zero or threat is armed, transition to attack state 0x20.
// ---------------------------------------------------------------------------
pub fn fn_s_person_give_warning(entity: *mut u8) {
    unsafe {
        let ebx = entity;
        *ebx.add(0x54) = 0;

        let threat_off = *(ebx.add(0x2a) as *const u16) as usize;
        let threat = LEVEL_THINGS_BASE.add(threat_off);
        let timer = *(ebx.add(0x2c) as *const u16);

        if timer == 0 || *(threat.add(0x44) as *const u16) != 0 {
            *ebx.add(0x19) = 0x20; // ATTACK
            return;
        }

        let tgt_y = *(threat.add(0x6) as *const i16) as i32;
        let ent_y = *(ebx.add(0x6) as *const i16) as i32;
        let tgt_x = *(threat.add(0x4) as *const i16) as i32;
        let ent_x = *(ebx.add(0x4) as *const i16) as i32;
        *ebx.add(0x1a) = get_angle(tgt_x.wrapping_sub(ent_x), tgt_y.wrapping_sub(ent_y));

        *(ebx.add(0x2c) as *mut u16) = timer.wrapping_sub(1);

        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x337b0  fn_S_PERSON_FOLLOW_AND_ATTACK
//
// Follow a target (field_0x2a) and attack it. Handles vehicle-mounted and
// on-foot targets separately, uses i_can_see_and_shoot_* for LOS checks
// and person_use_weapon to fire. Timer at field_0x42 gates attack frequency.
// ---------------------------------------------------------------------------
pub fn fn_s_person_follow_and_attack(entity: *mut u8) {
    use crate::syndre::funcs_20000::{animate_model, goto_angle};
    use crate::syndre::funcs_10000::getrdist;
    unsafe {
        let ebx = entity;
        let si = *(ebx.add(0x2a) as *const u16);
        let esi = LEVEL_THINGS_BASE.add(si as usize);

        // Face the target
        let tgt_y = *(esi.add(0x6) as *const i16) as i32;
        let ent_y = *(ebx.add(0x6) as *const i16) as i32;
        let tgt_x = *(esi.add(0x4) as *const i16) as i32;
        let ent_x = *(ebx.add(0x4) as *const i16) as i32;
        *ebx.add(0x1a) = get_angle(tgt_x.wrapping_sub(ent_x), tgt_y.wrapping_sub(ent_y));
        *ebx.add(0x46) = 0;

        // If timer set and target in z-band: keep timer at -1 (continuous fire)
        let weapon_ref = *(ebx.add(0x3a) as *const u16);
        if weapon_ref != 0 && (*esi.add(0x0b) & 0x1) == 0 {
            let zdiff = ac_abs(
                (*(esi.add(0x8) as *const i16) as i32)
                    .wrapping_sub(*(ebx.add(0x8) as *const i16) as i32)
            );
            if zdiff < 0x100 {
                *(ebx.add(0x42) as *mut i16) = -1i16;
            }
        }

        // Choose best weapon; bail to RELOAD if empty
        let best = choose_best_weapon(ebx, 0);
        let ebp = LEVEL_THINGS_BASE.add(best as usize);
        *(ebx.add(0x44) as *mut u16) = best;
        if (*(ebp.add(0x14) as *const i16)) < 0 {
            *(ebx.add(0x44) as *mut u16) = 0;
            *ebx.add(0x46) = 0;
            *ebx.add(0x19) = 0x1f;
            return;
        }

        let vdx0 = *(esi.add(0x24) as *const u16);

        if vdx0 != 0 {
            // --- Target is in a vehicle chain ---
            let mut vdx = vdx0;
            let mut target_veh: *mut u8 = LEVEL_THINGS_BASE.add(vdx0 as usize);
            loop {
                if vdx == 0 { break; }
                let vthing = LEVEL_THINGS_BASE.add(vdx as usize);
                let vtype = *vthing.add(0x18);
                if vtype == 1 {
                    vdx = *(vthing.add(0x24) as *const u16);
                } else if vtype == 2 {
                    target_veh = vthing;
                    break;
                } else {
                    break; // unreachable in practice
                }
            }

            let dy_v = (*(target_veh.add(0x6) as *const i16) as i32)
                .wrapping_sub(*(ebx.add(0x6) as *const i16) as i32);
            let dx_v = (*(target_veh.add(0x4) as *const i16) as i32)
                .wrapping_sub(*(ebx.add(0x4) as *const i16) as i32);
            let dist_v = getrdist(dx_v as i16, dy_v as i16);

            if (dist_v as u16) < 0x80 {
                // Too close: back off
                let spd = get_person_speed(ebx, *ebx.add(0x55) as i32);
                *ebx.add(0x54) = spd as u8;
                let ang = (*ebx.add(0x1a)).wrapping_add(0x80);
                goto_angle(spd as i16, ang);
                person_colide(ebx);
            } else {
                let wtype = *ebp.add(0x19) as usize;
                let range = DATA_5A6C2[wtype] as i32;
                let perception = get_person_perception(ebx, range);

                let can_see = i_can_see_and_shoot_vehicle(ebx, target_veh, perception);
                if can_see == target_veh {
                    let vz = (*(target_veh.add(0x8) as *const i16) as i32).wrapping_add(0x80);
                    let vy = *(target_veh.add(0x6) as *const i16) as i32;
                    let vx = *(target_veh.add(0x4) as *const i16) as i32;
                    person_use_weapon(ebx, vx, vy, vz);
                    let adrn = get_person_adrenlin(ebx, 0x32);
                    *ebx.add(0x58) = 0xd;
                    *(ebx.add(0x2c) as *mut u16) = (0x32i32 - adrn) as u16;
                    return;
                }

                if (dist_v as u32) >= 0x300 {
                    let spd = get_person_speed(ebx, *ebx.add(0x55) as i32);
                    *ebx.add(0x54) = spd as u8;
                    let ang = *ebx.add(0x1a);
                    goto_angle(spd as i16, ang);
                    if person_colide(ebx) != 0 {
                        quick_decide_on_hug_direction(ebx, 0xa);
                    }
                }
            }
            animate_model(ebx);
        } else {
            // --- Target is on foot ---
            let dy_f = (*(esi.add(0x6) as *const i16) as i32)
                .wrapping_sub(*(ebx.add(0x6) as *const i16) as i32);
            let dx_f = (*(esi.add(0x4) as *const i16) as i32)
                .wrapping_sub(*(ebx.add(0x4) as *const i16) as i32);
            let dist_f = getrdist(dx_f as i16, dy_f as i16);

            if (dist_f as u16) < 0x80 {
                let spd = get_person_speed(ebx, *ebx.add(0x55) as i32);
                *ebx.add(0x54) = spd as u8;
                let ang = (*ebx.add(0x1a)).wrapping_add(0x80);
                goto_angle(spd as i16, ang);
                person_colide(ebx);
                animate_model(ebx);
            } else {
                let wtype = *ebp.add(0x19) as usize;
                let range = DATA_5A6C2[wtype] as i32;
                let perception = get_person_perception(ebx, range);

                let can_see = i_can_see_and_shoot_person(ebx, esi, perception);
                if !can_see.is_null() {
                    // Check if target is shielding (weapon state 0x11)
                    let armed = *(esi.add(0x44) as *const u16);
                    let shielding = armed != 0 && {
                        let armed_thing = LEVEL_THINGS_BASE.add(armed as usize);
                        *armed_thing.add(0x19) == 0x11
                    };
                    if !shielding {
                        let tz = (*(esi.add(0x8) as *const i16) as i32).wrapping_add(0x80);
                        let ty2 = *(esi.add(0x6) as *const i16) as i32;
                        let tx2 = *(esi.add(0x4) as *const i16) as i32;
                        person_use_weapon(ebx, tx2, ty2, tz);
                        let adrn = get_person_adrenlin(ebx, 0x32);
                        *ebx.add(0x58) = 0xd;
                        *(ebx.add(0x2c) as *mut u16) = (0x32i32 - adrn) as u16;
                        return;
                    }
                } else if (dist_f as u32) >= 0x300 {
                    let spd = get_person_speed(ebx, *ebx.add(0x55) as i32);
                    *ebx.add(0x54) = spd as u8;
                    let ang = *ebx.add(0x1a);
                    goto_angle(spd as i16, ang);
                    if person_colide(ebx) != 0 {
                        quick_decide_on_hug_direction(ebx, 0xa);
                    }
                }
                animate_model(ebx);
            }
        }

        // jump_33af8: per-tick fire cooldown timer
        let si_timer = *(ebx.add(0x42) as *const i16);
        if si_timer < 0 {
            let adrn = get_person_adrenlin(ebx, 0x32);
            *ebx.add(0x19) = 0xd; // WAIT_FOR_VEHICLE
            *(ebx.add(0x2c) as *mut u16) = (0x32i32 - adrn) as u16;
        } else if si_timer > 0 {
            *(ebx.add(0x42) as *mut i16) = si_timer.wrapping_sub(1);
        }

        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x33b40  fn_S_PERSON_DROWN
//
// Drowning state: play death animation; when it finishes, mark entity DEAD.
// ---------------------------------------------------------------------------
pub fn fn_s_person_drown(entity: *mut u8) {
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let ebx = entity;
        *ebx.add(0x54) = 0;
        let done = animate_model(ebx);
        if done != 0 {
            *ebx.add(0x19) = 0x1a; // DEAD
            *ebx.add(0xa) = *ebx.add(0xa) | 0x1;
        }
    }
}

// ---------------------------------------------------------------------------
// 0x3a0d0  fatal_weapon
//
// Returns the "fatality rating" of an entity's currently selected weapon
// (DATA_5A686[weapon.type]). Returns 0 if no valid weapon equipped.
// ---------------------------------------------------------------------------
pub fn fatal_weapon(entity: *const u8) -> u16 {
    unsafe {
        let offset = *(entity.add(0x44) as *const u16) as usize;
        let weapon_ptr = LEVEL_THINGS_BASE.add(offset);
        if weapon_ptr < LEVEL_WEAPONS {
            return 0;
        }
        let wtype = (*weapon_ptr.add(0x19) as usize) & 0xff;
        DATA_5A686[wtype.min(9)] as u16
    }
}

// ---------------------------------------------------------------------------
// 0x316d0  agent_check_arc_for_enemy
//
// Scan all active persons looking for a hostile target visible from `entity`.
// Skips persons in the same group-of-8 as `entity`. Calls
// i_can_see_and_shoot_person(entity, candidate, perception) on each
// qualifying person. Returns first visible enemy or null.
//
// Qualifying criteria: type==1, has weapon OR flag 0x2 in field_0x1c,
// field_0xa bits 0x109 clear (not dead/inactive), field_0x20 == 0.
// ---------------------------------------------------------------------------
pub fn agent_check_arc_for_enemy(
    entity: *mut u8,
    perception: i32,
    intelligence: i32,
) -> *mut u8 {
    unsafe {
        if intelligence <= 0 {
            return std::ptr::null_mut();
        }

        // Compute entity's group start index (entity_idx rounded down to multiple of 8)
        let offset_bytes = (entity as usize).wrapping_sub(LEVEL_THINGS_BASE as usize) as u16;
        let entity_idx = (offset_bytes as usize) / 0x5c;
        let group_start = (entity_idx & !7) as u16; // matches `and $0xf8, %al`

        let mut ebx = LEVEL_PEOPLE;
        if ebx >= LAST_PERSON {
            return std::ptr::null_mut();
        }

        let mut si: u16 = 0; // scan index
        let di = group_start;

        loop {
            // Skip same group: di <= si < di+8
            let in_group = si >= di && si < di.wrapping_add(8);
            if !in_group {
                // Check if this person is a valid hostile target
                if *ebx.add(0x18) == 1 {  // type == PERSON
                    let has_weapon = *(ebx.add(0x44) as *const u16) != 0;
                    let is_faction = (*ebx.add(0x1c) & 0x2) != 0;
                    if has_weapon || is_faction {
                        let flags_a = *(ebx.add(0xa) as *const u16);
                        if (flags_a & 0x109) == 0 {   // not dead/inactive
                            let following = *(ebx.add(0x20) as *const u16);
                            if following == 0 {
                                let result = i_can_see_and_shoot_person(
                                    entity,
                                    ebx,
                                    perception,
                                );
                                if result == ebx {
                                    return ebx;
                                }
                            }
                        }
                    }
                }
            }

            ebx = ebx.add(0x5c);
            si = si.wrapping_add(1);
            if ebx >= LAST_PERSON {
                break;
            }
        }
        std::ptr::null_mut()
    }
}

// ---------------------------------------------------------------------------
// 0x34030  fn_S_PERSON_BEING_PERSUADED
//
// Target of a persuasion attempt: play animation; when it ends, yield.
// ---------------------------------------------------------------------------
pub fn fn_s_person_being_persuaded(entity: *mut u8) {
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let ebx = entity;
        *ebx.add(0x54) = 0;
        let done = animate_model(ebx);
        if done != 0 {
            new_state_person(ebx);
        }
    }
}

// ---------------------------------------------------------------------------
// 0x33b80  fn_S_PERSON_WAIT_TO_CROSS_ROAD
//
// Stands at a road crossing until check_for_on_coming_cars returns clear,
// then calls new_state_person. Always calls affect_person.
// ---------------------------------------------------------------------------
pub fn fn_s_person_wait_to_cross_road(entity: *mut u8) {
    unsafe {
        let ebx = entity;
        let angle_idx = *ebx.add(0x1a) as usize;
        let spd_i32 = 0x100_i32; // shl $0x8 then sar $0x8 → identity but keeps precision

        // Compute projected position using sin/cos tables at current angle
        let sin_val = *(DATA_5AB60.as_ptr().add(angle_idx * 2) as *const i16) as i32;
        let cos_val = *(DATA_5AD60.as_ptr().add(angle_idx * 2) as *const i16) as i32;
        // Assembly multiplies by 0x100 then sars by 8 (net: identity, but
        // uses the sin/cos table with an integer shift to get tile-scale delta)
        let tx = (DATA_60B28 as i32).wrapping_add((sin_val.wrapping_mul(spd_i32)) >> 8);
        let ty = (DATA_60B2A as i32).wrapping_add((cos_val.wrapping_mul(spd_i32)) >> 8);
        let tz = DATA_60B2C as i32;

        if check_for_on_coming_cars(tx, ty, tz) == 0 {
            *ebx.add(0xa) = *ebx.add(0xa) & 0xf7;
            new_state_person(ebx);
        }

        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x33c00  fn_S_PERSON_WALK_OFF_ROAD
//
// Walk perpendicular to a road until off it. Checks alignment (field_0x5b
// saves the expected angle), then either transitions back or adjusts angle.
// ---------------------------------------------------------------------------
pub fn fn_s_person_walk_off_road(entity: *mut u8) {
    use crate::syndre::funcs_20000::{animate_model, goto_angle};
    unsafe {
        let ebx = entity;
        *ebx.add(0x54) = 0x20;

        let tx = DATA_60B28 as i32;
        let ty = DATA_60B2A as i32;
        let tz = DATA_60B2C as i32 - 1;

        if there_is_a_road_here(tx, ty, tz) == 0 {
            // Off road: clear flag, switch state
            *ebx.add(0xa) = *ebx.add(0xa) & 0xf7;
            new_state_person(ebx);
        } else {
            let al = *ebx.add(0x1a);
            let dy_raw = DATA_60B2A as i32; // dx in asm is actually DATA_60B2A
            let cx_raw = DATA_60B28 as i32;

            // Depending on heading, check perpendicular clearance and deviation
            let mut transitioned = false;
            if al == 0x00 {
                if dy_raw >= 0x40 {
                    let saved = *ebx.add(0x5b) as i8;
                    let diff = ac_abs(0_i32 - saved as i32);
                    if diff as u8 <= 0x40 {
                        *ebx.add(0xa) = *ebx.add(0xa) & 0xf7;
                        new_state_person(ebx);
                        transitioned = true;
                    } else {
                        *ebx.add(0x19) = 0x26; // state = WALK_OFF_ROAD
                        *ebx.add(0x1a) = (*ebx.add(0x1a)).wrapping_add(0x80);
                    }
                }
            } else if al == 0x40 {
                if cx_raw >= 0x40 && cx_raw <= 0xc0 {
                    let saved = *ebx.add(0x5b) as i8;
                    let diff = ac_abs(0x40_i32 - saved as i32);
                    if diff as u8 <= 0x40 {
                        *ebx.add(0xa) = *ebx.add(0xa) & 0xf7;
                        new_state_person(ebx);
                        transitioned = true;
                    } else {
                        *ebx.add(0x19) = 0x26;
                        *ebx.add(0x1a) = (*ebx.add(0x1a)).wrapping_add(0x80);
                    }
                }
            } else if al == 0x80 {
                if cx_raw <= 0xc0 {
                    let saved = *ebx.add(0x5b) as i8;
                    let diff = ac_abs(0_i32 - saved as i32);
                    if diff as u8 <= 0x40 {
                        *ebx.add(0xa) = *ebx.add(0xa) & 0xf7;
                        new_state_person(ebx);
                        transitioned = true;
                    } else {
                        *ebx.add(0x19) = 0x26;
                        *ebx.add(0x1a) = (*ebx.add(0x1a)).wrapping_add(0x80);
                    }
                }
            } else if al == 0xc0 {
                if cx_raw >= 0x40 && cx_raw <= 0xc0 {
                    let saved = *ebx.add(0x5b) as i8;
                    let diff = ac_abs(0x40_i32 - saved as i32);
                    if diff as u8 <= 0x40 {
                        *ebx.add(0xa) = *ebx.add(0xa) & 0xf7;
                        new_state_person(ebx);
                        transitioned = true;
                    } else {
                        *ebx.add(0x19) = 0x26;
                        *ebx.add(0x1a) = (*ebx.add(0x1a)).wrapping_add(0x80);
                    }
                }
            }
            let _ = transitioned;
        }

        let cur_angle = *ebx.add(0x1a);
        let cur_speed = *ebx.add(0x54);
        goto_angle(cur_speed as i16, cur_angle);

        if person_colide(ebx) != 0 {
            quick_decide_on_hug_direction(ebx, 0xa);
        }

        animate_model(ebx);
        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x33dc0  fn_S_PERSON_WAIT_FOR_TRIGGER
//
// Waits for an allied person to appear in a 5-column strip at the entity's
// goto_x/goto_y + field_0x42 row offset. When found, calls new_state_person.
// Uses field_0x42 as a row counter (0-4 cycling), capped at 5.
// ---------------------------------------------------------------------------
pub fn fn_s_person_wait_for_trigger(entity: *mut u8) {
    unsafe {
        let ebx = entity;
        *ebx.add(0x54) = 0;

        // Timer: clamp to [0,5], then increment for next tick
        let dx = *(ebx.add(0x42) as *const i16);
        if dx > 5 || dx < 0 {
            *(ebx.add(0x42) as *mut i16) = 0;
        } else {
            *(ebx.add(0x42) as *mut i16) = dx.wrapping_add(1);
        }

        if !LEVEL_MAPWHO.is_null() {
            let row_off = *(ebx.add(0x42) as *const i16) as i32;
            let goto_y = *(ebx.add(0x30) as *const i16) as i32;
            let goto_x = *(ebx.add(0x2e) as *const i16) as i32;
            let trigger_z = *(ebx.add(0x32) as *const i16);

            'scan: for cx in 0i32..5 {
                let tile_idx = (goto_x + cx) + (goto_y + row_off) * 128;
                if tile_idx < 0 { continue; }
                let mut ax = *(LEVEL_MAPWHO.add(tile_idx as usize * 2) as *const u16) as usize;
                loop {
                    if ax == 0 { break; }
                    let thing = LEVEL_THINGS_BASE.add(ax);
                    if *thing.add(0x18) == 1 && (*thing.add(0x1c) & 0x2) != 0 {
                        let tz = *(thing.add(0x8) as *const i16);
                        if tz == trigger_z {
                            new_state_person(ebx);
                            break 'scan;
                        }
                    }
                    ax = *(thing.add(0x0) as *const u16) as usize;
                }
            }
        }

        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x33e60  fn_S_PERSON_WAIT_FOR_TRAIN
//
// Look for a stopped vehicle (type 2, state 0x9 or 0xa) in the mapwho tile
// ahead of the entity. If found and has room, board it. Otherwise countdown
// timer (field_0x42); at zero flip angle and new_state_person.
// ---------------------------------------------------------------------------
pub fn fn_s_person_wait_for_train(entity: *mut u8) {
    unsafe {
        let esi = entity;
        *esi.add(0x54) = 0;

        // Compute tile ahead using sin/cos of current angle
        let angle_idx = *esi.add(0x1a) as usize;
        let ey = *(esi.add(0x6) as *const i16) as i32;
        let ex = *(esi.add(0x4) as *const i16) as i32;
        let sin_val = *(DATA_5AB60.as_ptr().add(angle_idx * 2) as *const i16) as i32;
        let cos_val = *(DATA_5AD60.as_ptr().add(angle_idx * 2) as *const i16) as i32;
        // asm: shl $0x8 then sar $0x8 → effectively shifts by 256 then right 8 = identity
        let nx = ex.wrapping_add(sin_val) & 0x7f00;
        let ny_masked = (ey.wrapping_add(cos_val)) & 0x7f00;
        let tile_x = (nx >> 8) & 0x7f;
        let tile_y = (ny_masked >> 1) & !0x7f | ((ny_masked >> 8) & 0x7f);
        // Simplified: use same mapwho index calculation as elsewhere
        let tile_y2 = ((ey.wrapping_add(cos_val)) >> 8) & 0x7f;
        let tile_x2 = ((ex.wrapping_add(sin_val)) >> 8) & 0x7f;
        let mapwho_idx = tile_y2 * 128 + tile_x2;
        let _ = (tile_x, tile_y); // suppress unused

        let mut found_vehicle = false;
        if !LEVEL_MAPWHO.is_null() && mapwho_idx >= 0 {
            let mut ax = *(LEVEL_MAPWHO.add(mapwho_idx as usize * 2) as *const u16) as usize;
            'outer: loop {
                if ax == 0 { break; }
                let ebx = LEVEL_THINGS_BASE.add(ax);
                if *ebx.add(0x18) == 2 {  // type == VEHICLE
                    let vstate = *ebx.add(0x19);
                    if vstate == 0x9 || vstate == 0xa {
                        // Walk forward pointer chain (field_0x20) to find last passenger slot
                        let mut veh = ebx;
                        let mut dx2 = *(veh.add(0x20) as *const u16);
                        if dx2 != 0 {
                            loop {
                                let next = LEVEL_THINGS_BASE.add(dx2 as usize);
                                let nd = *(next.add(0x20) as *const u16);
                                if nd == 0 { veh = next; break; }
                                dx2 = nd;
                                veh = next;
                            }
                        }
                        // Check if last vehicle slot has field_0x1c == 0 (free slot)
                        let cx = *(veh.add(0x1c) as *const u16);
                        if cx != 0 {
                            let cx_thing = LEVEL_THINGS_BASE.add(cx as usize);
                            if *cx_thing.add(0x19) == 0x2a { break 'outer; }
                        }
                        // Board the vehicle
                        let veh_off = (ebx as usize).wrapping_sub(LEVEL_THINGS_BASE as usize) as u16;
                        *esi.add(0x58) = 0x6;  // desired = MOVE_INTO_VEHICLE
                        *(esi.add(0x2c) as *mut u16) = veh_off;
                        new_state_person(esi);
                        found_vehicle = true;
                        break;
                    }
                }
                ax = *(ebx.add(0x0) as *const u16) as usize;
            }
        }

        if !found_vehicle {
            // Countdown timer; at zero: flip angle and give up
            let timer = *(esi.add(0x42) as *const i16);
            let new_timer = timer.wrapping_sub(1);
            *(esi.add(0x42) as *mut i16) = new_timer;
            if timer == 0 {
                let ah = *esi.add(0x1a);
                *esi.add(0x1a) = ah.wrapping_add(0x80);
                new_state_person(esi);
            }
        }

        let new_state = affect_person(esi) as u8;
        *esi.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x33f70  fn_S_PERSON_ALLOW_PASSENGERS
//
// Vehicle crew waits while passengers board. Counts down field_0x2c (max
// 0x1f4); when it expires or was zero, switch to state 0xd (WAIT_FOR_VEH).
// ---------------------------------------------------------------------------
pub fn fn_s_person_allow_passengers(entity: *mut u8) {
    unsafe {
        let ebx = entity;
        *ebx.add(0x54) = 0;

        let dx = *(ebx.add(0x2c) as *const u16);
        if dx == 0 || dx > 0x1f4 {
            *(ebx.add(0x2c) as *mut u16) = 0x32;
            *ebx.add(0x19) = 0xd; // WAIT_FOR_VEHICLE
        } else {
            *(ebx.add(0x2c) as *mut u16) = dx.wrapping_sub(1);
        }

        let new_state = affect_person(ebx) as u8;
        *ebx.add(0x19) = new_state;
    }
}

// ---------------------------------------------------------------------------
// 0x33fb0  fn_S_PERSON_USE_WEAPON
//
// Fire animation state. Clears field_0xa bit3, calls affect_person; if state
// changed, return immediately. Otherwise animate, and when done:
// call auto_weapon; if it returns non-zero, end via new_state_person +
// set DATA_5E12C=1; else set field_0xa bit3 (continue firing).
// ---------------------------------------------------------------------------
pub fn fn_s_person_use_weapon(entity: *mut u8) {
    use crate::syndre::funcs_20000::animate_model;
    unsafe {
        let ebx = entity;
        *ebx.add(0x54) = 0;
        *ebx.add(0xa) = *ebx.add(0xa) & 0xf7; // clear bit 3

        let affect_result = affect_person(ebx) as u16 & 0xffff;
        let cur_state = *ebx.add(0x19) as u16;
        if cur_state != affect_result {
            *ebx.add(0x19) = affect_result as u8;
            return;
        }

        let anim_done = animate_model(ebx);
        if anim_done != 0 || auto_weapon(ebx) != 0 {
            which_frame_person(ebx);
            new_state_person(ebx);
            DATA_5E12C = 1;
        } else {
            *ebx.add(0xa) |= 0x08;
        }
    }
}

// ---------------------------------------------------------------------------
// 0x34110  move_people
//
// Per-tick simulation loop over all LEVEL_PEOPLE entries. Calls person_intel
// first, then for each active person: saves coords to DATA_60B28/2A/2C,
// runs person_on_block + optional adjust_bar_levels, dispatches the person's
// FSM state (field_0x19) via a match (originally a vtable), then runs the
// common epilogue: person_danger, danger counter decrement, weapon_is_empty
// check, and which_frame_person if DATA_5E12C == 0.
// ---------------------------------------------------------------------------
pub fn move_people_impl() {
    unsafe {
        person_intel();
        DATA_55300 = 0;

        let mut ebx = LEVEL_PEOPLE;
        if ebx.is_null() || ebx >= LAST_PERSON {
            return;
        }

        loop {
            DATA_5E12C = 0;

            if *ebx.add(0x18) != 0 {
                // Save entity coords to per-tick accumulators
                DATA_60B28 = *(ebx.add(0x4) as *const i16);
                DATA_60B2A = *(ebx.add(0x6) as *const i16);
                DATA_60B2C = *(ebx.add(0x8) as *const i16);

                person_on_block(ebx);

                if (*ebx.add(0x1c) & 0x2) != 0 {
                    adjust_bar_levels(ebx);
                }

                let fsm = *ebx.add(0x19);
                if fsm <= 0x2c {
                    match fsm {
                        0x00 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_stand(ebx); }
                        0x01 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_next_command(ebx); }
                        0x02 | 0x03 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_goto_point(ebx); }
                        0x04 | 0x05 => {
                            if *(ebx.add(0x20) as *const u16) != 0 {
                                *ebx.add(0x1d) |= 0x08;
                            } else {
                                *ebx.add(0x1d) &= 0xf7;
                            }
                            fn_s_person_goto_structure(ebx);
                        }
                        0x06 => { *ebx.add(0x1d) |= 0x08;  fn_s_person_move_into_vehicle(ebx); }
                        0x07 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_move_out_of_vehicle(ebx); }
                        0x08 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_passenger(ebx); }
                        0x09 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_pickup_weapon(ebx); }
                        0x0a => { *ebx.add(0x1d) &= 0xf7; fn_s_person_drop_weapon(ebx); }
                        0x0b => { *ebx.add(0x1d) &= 0xf7; fn_s_person_select_weapon(ebx); }
                        0x0c => { *ebx.add(0x1d) &= 0xf7; fn_s_person_wait_for_model(ebx); }
                        0x0d => { *ebx.add(0x1d) &= 0xf7; fn_s_person_wait_for_time(ebx); }
                        0x0e => { fn_s_person_hug_right(ebx); }
                        0x0f => { fn_s_person_hug_left(ebx); }
                        0x10 => { *ebx.add(0x1d) &= 0xf7; } // no FSM call
                        0x11 => { *ebx.add(0x1d) |= 0x08;  fn_s_person_hit_by_laser(ebx); }
                        0x12 => { *ebx.add(0x1d) |= 0x08;  fn_s_person_hit_by_bullet(ebx); }
                        0x13 => { *ebx.add(0x1d) |= 0x08;  fn_s_person_hit_by_vehicle(ebx); }
                        0x14 => { *ebx.add(0x1d) |= 0x08;  fn_s_person_hit_by_fire(ebx); }
                        0x15 => { *ebx.add(0x1d) |= 0x08;  fn_s_person_hit_by_explosion(ebx); }
                        0x16 => { *ebx.add(0x1d) |= 0x08;  fn_s_person_fly_back(ebx); }
                        0x17 => { *ebx.add(0x1d) |= 0x08;  fn_s_person_on_fire(ebx); }
                        0x18 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_dying(ebx); }
                        0x19 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_dying_on_fire(ebx); }
                        0x1a => { *ebx.add(0x1d) &= 0xf7; fn_s_person_dead(ebx); }
                        0x1b => { *ebx.add(0x1d) &= 0xf7; fn_s_person_dead_on_fire(ebx); }
                        0x1c => { *ebx.add(0x1d) &= 0xf7; fn_s_person_guard_area(ebx); }
                        0x1d => { *ebx.add(0x1d) &= 0xf7; fn_s_person_wander(ebx); }
                        0x1e => { *ebx.add(0x1d) |= 0x08;  fn_s_person_persuaded(ebx); }
                        0x1f => { *ebx.add(0x1d) |= 0x08;  fn_s_person_runaway(ebx); }
                        0x20 => { *ebx.add(0x1d) |= 0x08;  fn_s_person_follow_and_attack(ebx); }
                        0x21 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_drown(ebx); }
                        0x22 => { *ebx.add(0x1d) |= 0x08;  fn_s_person_give_warning(ebx); }
                        0x23..=0x25 => { *ebx.add(0x1d) &= 0xf7; } // reserved, no call
                        0x26 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_wait_to_cross_road(ebx); }
                        0x27 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_walk_off_road(ebx); }
                        0x28 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_wait_for_trigger(ebx); }
                        0x29 => { *ebx.add(0x1d) &= 0xf7; fn_s_person_wait_for_train(ebx); }
                        0x2a => { *ebx.add(0x1d) &= 0xf7; fn_s_person_allow_passengers(ebx); }
                        0x2b => { fn_s_person_use_weapon(ebx); }
                        0x2c => { fn_s_person_being_persuaded(ebx); }
                        _ => {}
                    }
                }

                // Common epilogue: danger counter, weapon check, animation frame
                person_danger(ebx);
                let dl = *ebx.add(0x46);
                if dl > 0 {
                    *ebx.add(0x46) = dl - 1;
                }
                if (*ebx.add(0x1c) & 0x2) != 0 && weapon_is_empty(ebx) != 0 {
                    *ebx.add(0x46) = 0;
                }
                if DATA_5E12C == 0 {
                    which_frame_person(ebx);
                }
            }

            ebx = ebx.add(0x5c);
            if ebx >= LAST_PERSON {
                break;
            }
        }
    }
}
