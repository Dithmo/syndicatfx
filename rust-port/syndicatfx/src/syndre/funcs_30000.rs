// syndre/funcs_30000.rs — Translated x86 functions at addresses 0x30000–0x3FFFF.
//
// This range contains: menu_select (0x37600), level_finished (0x37f80),
// init_sound (0x385c0), and supporting UI + animation functions.

use crate::globals::*;
use crate::syndre::data::*;

// Untranslated functions called from this range
extern "C" {
    fn ac_LbDataLoadAll(files: *mut u8) -> i32;
    fn ac_sound_bank_setup();
    fn weapons_in_weight(entity: *const u8) -> i32;
    fn ac_abs(x: i32) -> i32;
    fn can_i_persuad_you(entity: *mut u8, target: *mut u8) -> u16;
    fn i_cant_walk_in_this_direction(entity: *mut u8, direction: u32) -> u16;
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
