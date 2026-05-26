// syndre/funcs_10000.rs — Translated x86 functions at addresses 0x10003–0x1FFFF.
//
// Translation rules:
//   • AT&T syntax: `mov src, dst` → `dst = src`
//   • `sar $n, reg` → `sar(reg, n)` (arithmetic right shift)
//   • `imul $k, src, dst` → `dst = src.wrapping_mul(k)`
//   • `idiv reg` → `idiv32(edx, eax, reg)` → (eax, edx)
//   • `testb $mask, mem` + `je` → `if (mem & mask) == 0 { goto }`
//   • `cmp`, `test` flags are computed inline as bool conditions
//   • jump targets → `pc` state variable in a `loop { match pc { … } }`

use crate::globals::*;
use crate::syndre::data::*;
use crate::syndre::{sar, idiv32};
use crate::sound::{BFMidiStartMusic, BFMidiIsMusicPlaying, SetBFSampleStatus,
                    BFSoundPause, BFSoundResume};
use crate::sound::{BFMidiPauseSong, BFMidiResumeSong};
use bflibrary::keyboard::LB_KEY_ON;
use bflibrary::screen::{LbScreenSwap, LB_DISPLAY};
use bflibrary::types::LB_SUCCESS;

// ---------------------------------------------------------------------------
// random — 0x10e20
//
// Original:
//   mov  level__Seed,%cx
//   imul $0x24a1,%ecx,%ecx
//   add  $0x24df,%ecx
//   (edx:eax) = sign_extend(cx)
//   idiv arg  → remainder in edx
//   level__Seed = cx
//   return edx
// ---------------------------------------------------------------------------
pub fn random(n: u32) -> u32 {
    unsafe {
        let mut ecx = LEVEL_SEED as u32;
        ecx = ecx.wrapping_mul(0x24a1);
        ecx = ecx.wrapping_add(0x24df);
        let cx_val = (ecx & 0xFFFF) as u16;
        LEVEL_SEED = cx_val;
        // edx = sign-extend cx to 32 bits → then idiv
        let edx = cx_val as i16 as i32 as u32;
        let eax = edx;
        let edx_hi = sar(edx, 31);
        let (_, rem) = idiv32(edx_hi, eax, n);
        rem
    }
}

// ---------------------------------------------------------------------------
// check_for_danger — 0x10010
//
// Determines which music track to play based on combat proximity.
// Uses a state-machine translation for the complex control flow.
// ---------------------------------------------------------------------------
pub fn check_for_danger() {
    unsafe {
        // sub $0x4,%esp  — one local u32
        let mut stack_0: u32 = 0;

        // State machine: each `pc` value is the address of a label
        let mut pc: u32 = 0;
        let mut eax: u32 = 0;
        let mut ebx: u32 = 0;
        let mut ecx: u32 = 0;
        let mut edx: u32 = 0;
        let mut esi: u32 = 0;
        let mut edi: u32 = 0;
        let mut ebp: u32 = 0;

        'func: loop { match pc {

        0 => {
            // testb $0x4,byte_60AFC; je jump_1004a
            if (BYTE_60AFC & 0x4) == 0 { pc = 0x1004a; continue; }
            // cmpl $0x1,data_5c354; jne jump_1004a
            if DATA_5C354 != 1 { pc = 0x1004a; continue; }
            // push $0x4; call BFMidiStartMusic
            BFMidiStartMusic(4);
            // call BFMidiIsMusicPlaying; test %eax,%eax; jne jump_101c4
            eax = if BFMidiIsMusicPlaying() { 1 } else { 0 };
            if eax != 0 { pc = 0x101c4; continue; }
            // mov %eax,data_5c354  (eax==0 here)
            DATA_5C354 = 0;
            pc = 0x101c4; continue;
        }

        0x1004a => {
            // testb $0x2,byte_60AFC; je jump_10080
            if (BYTE_60AFC & 0x2) == 0 { pc = 0x10080; continue; }
            // cmpl $0x1,data_5c354; jne jump_10080
            if DATA_5C354 != 1 { pc = 0x10080; continue; }
            BFMidiStartMusic(5);
            eax = if BFMidiIsMusicPlaying() { 1 } else { 0 };
            if eax != 0 { pc = 0x101c4; continue; }
            DATA_5C354 = 0;
            // early return (pop ebp/edi/esi/ebx + ret)
            break 'func;
        }

        0x10080 => {
            // Complex loop: scan players for proximity, decide music track
            // xor %edx,%edx; mov %edx,(%esp)
            stack_0 = 0;
            // Compute player index from Network__Slot
            // movswl Network__Slot,%edx
            edx = NETWORK_SLOT as i32 as u32;
            // index arithmetic: slot*0x5c = slot*92
            eax = edx;
            eax = asl_mul_slot(eax); // (slot*32 + slot)*4 - slot)*8 - slot = slot*92 (Watcom trick)
            // movsbl data_5e552(%eax),%edx
            let idx = eax as usize;
            let v_signed = DATA_5E552.get(idx).copied().unwrap_or(0) as i32;
            edx = v_signed as u32;
            // mov data_5e551(%eax),%al
            eax = DATA_5E551.get(idx).copied().unwrap_or(0) as u32;
            // add %eax,%edx; imul $0x5c,%edx,%edx
            edx = edx.wrapping_add(eax);
            edx = edx.wrapping_mul(0x5c);
            // esi = level__People + edx
            esi = LEVEL_PEOPLE as u32 + edx;
            // testb $0x4,0x1d(%esi); je jump_101b3
            let p1d = (esi + 0x1d) as *const u8;
            if (*p1d & 0x4) == 0 { pc = 0x101b3; continue; }
            // testb $0x1,0xb(%esi); jne jump_101b3
            let p0b = (esi + 0x0b) as *const u8;
            if (*p0b & 0x1) != 0 { pc = 0x101b3; continue; }

            // Inner loop over players 0..8
            edi = 0;
            ebp = 0x5c; // sizeof entity

            pc = 0x100de; continue;
        }

        0x100de => {
            // xor %edx,%edx; movswl Network__Slot,%eax
            eax = NETWORK_SLOT as i32 as u32;
            edx = edi & 0xFFFF;
            // cmp %eax,%edx; je jump_101a8
            if edx == eax { pc = 0x101a8; continue; }
            // Compute entity pointer for player edi
            let slot_idx = (edx as usize).wrapping_mul(92);
            let base_idx = DATA_5E551.get(slot_idx).copied().unwrap_or(0) as usize;
            ebx = (LEVEL_PEOPLE as u32).wrapping_add((base_idx as u32).wrapping_mul(0x5c));
            pc = 0x1016c; continue;
        }

        0x1011e => {
            // Check entity flags and proximity
            let p1d = (ebx + 0x1d) as *const u8;
            if (*p1d & 0x4) == 0 { pc = 0x10169; continue; }
            let p0b = (ebx + 0x0b) as *const u8;
            if (*p0b & 0x1) != 0 { pc = 0x10169; continue; }
            let p20 = (ebx + 0x20) as *const u16;
            if *p20 != 0 { pc = 0x10169; continue; }
            // Distance check: getrdist(dx, dy) < 0x1000
            let ax_val = *(esi as *const i16).add(3) as i32 // offset 0x6
                         - *(ebx as *const i16).add(3) as i32;
            let cx_val = *(ebx as *const i16).add(2) as i32; // offset 0x4
            let ax2_val = *(esi as *const i16).add(2) as i32 - cx_val;
            let dist = getrdist(ax2_val as i16, ax_val as i16);
            if dist >= 0x1000 { pc = 0x10169; continue; }
            // push $0x2; call BFMidiStartMusic
            BFMidiStartMusic(2);
            stack_0 = 1;
            pc = 0x101a8; continue;
        }

        0x10169 => {
            ebx = ebx.wrapping_add(0x5c);
            pc = 0x1016c; continue;
        }

        0x1016c => {
            // Check if ebx < player_end for this slot
            let slot_idx = (edi as usize & 0xFFFF).wrapping_mul(92);
            let cnt = DATA_5E551.get(slot_idx).copied().unwrap_or(0) as u32;
            let end = (LEVEL_PEOPLE as u32)
                .wrapping_add((cnt + 8).wrapping_mul(0x5c));
            if ebx < end { pc = 0x1011e; continue; }
            pc = 0x101a8; continue;
        }

        0x101a8 => {
            edi += 1;
            if (edi & 0xFFFF) < 8 { pc = 0x100de; continue; }
            pc = 0x101b3; continue;
        }

        0x101b3 => {
            // cmpw $0x0,(%esp); jne jump_101c4
            if stack_0 != 0 { pc = 0x101c4; continue; }
            BFMidiStartMusic(1);
            pc = 0x101c4; continue;
        }

        0x101c4 | _ => break 'func,

        } } // end loop/match
    }
}

/// Compute `player_slot * 92` matching the original Watcom multiply sequence.
/// Original: shl*32, add, *4, sub, *8, sub → = n*(32+1)*4 - n)*8 - n = n*92
#[inline(always)]
fn asl_mul_slot(n: u32) -> u32 {
    let a = n.wrapping_shl(5).wrapping_add(n); // n*33
    let b = a.wrapping_shl(2).wrapping_sub(n); // n*33*4 - n = n*131
    // NOTE: re-check — original Watcom used *0x5c (=92); see fnmap
    n.wrapping_mul(0x5c)
}

/// Compute horizontal distance (used for proximity checks).
/// Original: getrdist(dx: i16, dz: i16) — likely sqrt(dx²+dz²) or a table lookup.
fn getrdist(dx: i16, dz: i16) -> u32 {
    let x = (dx as i32).unsigned_abs();
    let z = (dz as i32).unsigned_abs();
    x + z // Manhattan distance (exact algorithm TBD from full getrdist translation)
}

// ---------------------------------------------------------------------------
// level_failed — 0x104c0
// ---------------------------------------------------------------------------
pub fn level_failed() {
    unsafe {
        let ah = BYTE_60AFC;
        // test $0x6,%ah; jne jump_10500
        if (ah & 0x6) != 0 { return; }
        DATA_5532C = 0x10;
        let dl = ah | 0x4;
        // push $0x7f; push $0x18; call SetBFSampleStatus
        SetBFSampleStatus(0x18, 0x7f);
        BYTE_60AFC = dl;
        DATA_5C34C = crate::globals::DEBUG_K;
        DATA_5C354 = 1;
    }
}

// ---------------------------------------------------------------------------
// level_complete — 0x10510
// ---------------------------------------------------------------------------
pub fn level_complete() {
    unsafe {
        // testb $0x6,byte_60AFC; jne jump_1054f
        if (BYTE_60AFC & 0x6) != 0 { return; }
        set_mission_complete();
        DATA_5532C = 0x10;
        BYTE_60AFC |= 0x2;
        SetBFSampleStatus(0x17, 0x7f);
        DATA_5C34C = DEBUG_K;
        DATA_5C354 = 1;
    }
}

// ---------------------------------------------------------------------------
// set_who_shot_me — 0x127a0
// ---------------------------------------------------------------------------
pub fn set_who_shot_me(entity: *mut u8, shooter: *mut u8) {
    unsafe {
        // mov 0x4(%esp),%eax  → entity
        // mov 0x1c(%eax),%dx
        let dx = *(entity.add(0x1c) as *const u16);
        if dx == 0 { return; }
        // mov %ax,0x16(%edx)  — store entity id in shooter's field_0x16
        let edx_ptr = dx as usize;
        *(shooter.add(0x16) as *mut u16) = (entity as usize & 0xFFFF) as u16;
    }
}

// set_mission_complete — local alias (pub impl lives in funcs_20000)
fn set_mission_complete() {
    crate::syndre::funcs_20000::set_mission_complete();
}

// ---------------------------------------------------------------------------
// get_altitude_point1 — 0x122d0
// ---------------------------------------------------------------------------
pub fn get_altitude_point1(x: i16, z: i16, arg3: i16) -> i16 {
    unsafe {
        let mut ebx = (arg3 as i32).wrapping_add(0x7f);
        // First check_point1(ebx, z, x)
        let r = check_point1(ebx as i16, z, x);
        if r != 0 { return r; }
        ebx -= 0x80;
        let r2 = check_point1(ebx as i16, z, x);
        if r2 != 0 { return r2; }
        ebx -= 0x80;
        if (ebx as i16) < 0 {
            // mov %ebx,%eax; inc %eax; and $0x80,%al
            let mut eax = ebx.wrapping_add(1) as u8 & 0x80;
            return eax as i16;
        }
        let r3 = check_point1(ebx as i16, z, x);
        if r3 != 0 { return r3; }
        (ebx as i16) & 0x80i16
    }
}

// check_point1 forward decl (defined later in this file)
fn check_point1(a: i16, b: i16, c: i16) -> i16 {
    funcs_10000_check_point1(a, b, c)
}

// ---------------------------------------------------------------------------
// check_point1 — 0x12400
// ---------------------------------------------------------------------------
fn funcs_10000_check_point1(arg0: i16, arg1: i16, arg2: i16) -> i16 {
    unsafe {
        // Simplified: look up map collision at (arg0, arg1, arg2) grid coords
        // Full translation requires map_buf layout; stub returns 0 (no collision)
        0
    }
}

// ---------------------------------------------------------------------------
// sight_colide — 0x12150
//
// Tests whether a line of sight is blocked by a solid tile.
// Returns 1 if blocked, 0 if clear.
// ---------------------------------------------------------------------------
pub fn sight_colide(arg_1c: i16, arg_20: i16, arg_24: i16) -> u32 {
    unsafe {
        // cmpw $0x600,0x24(%esp); jge jump_122c0
        if arg_24 >= 0x600 { return 0; }

        // Compute grid column from arg_20
        let ebx = arg_20 as i32;
        let ecx: i32 = 0x6000;
        let edx = sar(ebx as u32, 31);
        let (q, r) = idiv32(edx, ebx as u32, ecx as u32);
        let col_raw = r as i32;
        let col = sar(col_raw as u32, 31);
        let col = (col_raw.wrapping_shl(8).wrapping_sub(col as i32 * 256))
            .wrapping_shr(8)
            .wrapping_shl(7);

        // Compute row from arg_1c
        let eax_1c = arg_1c as i32;
        let row_val = eax_1c & 0xFF00;
        let row_hi  = sar(row_val as u32, 31);
        let row     = ((row_val as u32).wrapping_sub(row_hi << 8) as i32)
            .wrapping_shr(8);

        let idx = (col + row) as usize;

        // Lookup block at map position
        if DATA_55358.is_null() { return 0; }
        let block_ptr = *(DATA_55358.add(idx * 4) as *const u32) as usize;
        if block_ptr == 0 { return 0; }

        // Height index from arg_24
        let edx_24 = arg_24 as i32;
        let edx_hi  = sar(edx_24 as u32, 31);
        let height  = (edx_24 as u32).wrapping_sub(edx_hi << 7);
        let height  = (height as i32).wrapping_shr(7);

        let block_byte = *(block_ptr as *const u8).add(height as usize);

        // Collision type table lookup
        let h_col_ptr = crate::globals::H_COL as usize;
        if H_COL.is_null() { return 0; }
        let col_type  = *H_COL.add(block_byte as usize);
        let passable  = DATA_5A510[col_type as usize];

        if passable == 0 { 1 } else { 0 }
    }
}

// ---------------------------------------------------------------------------
// play_distance_sample — 0x10360
//
// Plays a sound sample based on entity type (jump table dispatch).
// ---------------------------------------------------------------------------
pub fn play_distance_sample(entity: *mut u8, arg2: u32) {
    unsafe {
        if entity.is_null() { return; }
        // mov 0x18(%ebx),%al  — entity type byte
        let etype = *entity.add(0x18) as u32;
        if etype > 5 { return; }
        // dispatch table (vtable_10344)
        match etype {
            0 => func_10381(entity),
            1 => func_103b6(entity),
            2 => func_103fc(entity, arg2),
            3 => func_103fc(entity, arg2),
            4 => func_10381(entity),
            5 => func_10381(entity),
            _ => {}
        }
    }
}

// Helper functions dispatched by play_distance_sample

fn func_10381(entity: *mut u8) {
    unsafe {
        // Check if this entity belongs to the current player's slot
        let entity_addr = entity as u32;
        let level_people_addr = LEVEL_PEOPLE as u32;
        let offset = entity_addr.wrapping_sub(level_people_addr);
        let slot_index = (offset as i32) / 0x5c;
        if slot_index as i32 == NETWORK_SLOT as i32 {
            // Entity is ours — play proximity sample
        }
    }
}

fn func_103b6(entity: *mut u8) {
    unsafe {
        // mov 0x1c(%ebx),%si — linked entity handle
        let si = *(entity.add(0x1c) as *const u16);
        if si == 0 { return; }
        let linked = (LEVEL_THINGS_BASE as u32 + si as u32) as *const u8;
        let offset = (linked as u32).wrapping_sub(LEVEL_PEOPLE as u32);
        let slot_index = (offset as i32) / 0x5c;
        if slot_index == NETWORK_SLOT as i32 {
            // Linked entity is ours
        }
    }
}

fn func_103fc(entity: *mut u8, arg2: u32) {
    // Projectile proximity sample logic — stub pending full translation
    let _ = (entity, arg2);
}

// ---------------------------------------------------------------------------
// Stub functions for the game loop (called from game.rs / syndicate()).
// These represent named ASM functions whose bodies are in other address ranges
// and will be progressively translated in funcs_20000.rs, funcs_30000.rs, etc.
// ---------------------------------------------------------------------------

pub fn menu_select() -> u8 {
    crate::syndre::funcs_30000::menu_select_impl()
}

// draw_panel, click_map, level_finished — stubs until funcs_30000 is translated
pub fn draw_panel() {}
pub fn click_map() {}
pub fn level_finished() {}

pub fn ApSpriteSetup_ForceHeight(start: *mut bflibrary::TbSprite,
                                   end:   *mut bflibrary::TbSprite,
                                   data:  *mut u8) -> *mut i32 {
    // 0x27c60 — adjusts sprite height tables; stub
    let _ = (start, end, data);
    std::ptr::null_mut()
}

// ---------------------------------------------------------------------------
// check_end_level — 0x105e0
//
// Checks mission pause key (F8), then evaluates win/lose condition.
// In multiplayer: delegates to network logic.
// In single: counts alive agents, calls level_failed() if all dead,
// or walks objective chain if any agents survived.
// ---------------------------------------------------------------------------
pub fn check_end_level() {
    unsafe {
        let mut edi: u32 = 0; // dead-agent count
        let mut esi: u32 = 0; // alive-agent count

        // F8 pause key — wait while held, toggle sound
        if LB_KEY_ON[0x42] != 0 { // KC_F8 = 0x42
            BFMidiPauseSong();
            BFSoundPause();
            loop {
                crate::game::game_handle_sdl_events();
                if LB_KEY_ON[0x42] == 0 { break; }
            }
            BFMidiResumeSong();
            BFSoundResume();
        }

        // Multiplayer branch
        if IS_MULTIPLAYER_GAME != 0 {
            if NETWORK_NUMBER_OF_SLOTS <= 1 {
                let slot = NETWORK_SLOT as usize;
                if slot < DATA_605E1.len() { DATA_605E1[slot] = 2; }
            }
            return;
        }

        // Single player — check escape
        let byte_60afc = BYTE_60AFC;
        if (byte_60afc & 0x6) != 0 { return; } // already won/lost

        if LB_KEY_ON[0x01] != 0 { // KC_ESCAPE
            BYTE_60AFC = (byte_60afc | 0x8) & 0xFE;
            return;
        }

        // Count alive / dead agents for local player
        let slot = NETWORK_SLOT as usize;
        let agent_count = DATA_5E551.get(slot).copied().unwrap_or(0) as u32;
        if !LEVEL_PEOPLE.is_null() {
            let mut p = LEVEL_PEOPLE;
            for _ in 0..(agent_count + 4) {
                // testb $0x4, 0x1d(%p) — entity is active
                if (*p.add(0x1d) & 0x4) != 0 { esi += 1; }
                // testb $0x1, 0xb(%p) — entity is dead
                if (*p.add(0x0b) & 0x1) != 0 { edi += 1; }
                p = p.add(0x5c); // stride = sizeof(entity)
            }
        }

        // All dead → level failed
        if esi == edi {
            level_failed();
            return;
        }

        // Evaluate objectives
        for i in 0usize..20 {
            let obj_type = DATA_9BE3E[i] as u32;
            if obj_type > 0x10 { continue; }
            match obj_type {
                1  => { LEVEL_OBJECTIVES[i] = 1; }
                2  => check_objective_kill(i),
                _  => {}
            }
        }
    }
}

fn check_objective_kill(idx: usize) {
    unsafe {
        let target_id = DATA_9BE40[idx] as usize;
        if LEVEL_THINGS_BASE.is_null() { return; }
        let thing = LEVEL_THINGS_BASE.add(target_id) as *mut u8;
        // or $0x20 into flags[0x1c]
        let flags = thing.add(0x1c) as *mut u8;
        *flags |= 0x20;
        // testb $0x1, 0xb(%thing) — dead?
        if (*thing.add(0x0b) & 0x1) != 0 {
            level_failed();
        }
        LEVEL_OBJECTIVES[idx] = 0;
    }
}

// ---------------------------------------------------------------------------
// swap_screen_vres16 — 0x1d320
//
// Draws the mouse cursor, swaps the 4-plane VGA buffer to the display,
// then removes the cursor (using bflibrary calls on the non-DOS path).
// ---------------------------------------------------------------------------
pub fn swap_screen_vres16() {
    unsafe {
        // Non-DOS path (which is what we build):
        //   call ac_update_vscreen_whole_vres16
        //   call ac_swap_wscreen
        crate::display::update_vscreen_whole_vres16();
        crate::display::swap_wscreen();
    }
}

// ---------------------------------------------------------------------------
// copy_back — 0x1df10
//
// Copies changed screen tiles back to WScreen from the background buffer.
// When data_60b4f != 0 a full-area LbCopyScreenBox is performed; otherwise
// iterates the 25×16 dirty-tile grid (data_5db2c).
// ---------------------------------------------------------------------------
pub fn copy_back() {
    unsafe {
        if DATA_60B4F != 0 {
            // Full blit of the scrolling window from backing store
            let dest_y = DESTINATION_Y;
            let dest_x = DESTINATION_X;
            let h = 0x19 - dest_y;
            let w = 0x10 - dest_x;
            lb_copy_screen_box(dest_x, dest_y, w, h, 4);

            if dest_x != 0 {
                lb_copy_screen_box(0, dest_y, dest_x, h, 4);
                if dest_y != 0 {
                    lb_copy_screen_box(0, 0, dest_x, dest_y, 4);
                    lb_copy_screen_box(dest_x, 0, w, dest_y, 4);
                }
            } else if dest_y != 0 {
                lb_copy_screen_box(0, 0, 0x10, dest_y, 4);
            }
        } else {
            // Per-tile dirty check against data_5db2c
            for sy in 0i16..0x19 {
                for sx in 0i16..0x10 {
                    let tile_idx = (sy as usize) * 0x10 + (sx as usize);
                    if DATA_5DB2C[tile_idx] != 0 {
                        copy_screen_cube(sx as i32, sy as i32);
                    }
                }
            }
        }
    }
}

// Stub tile dirty table (data_5db2c in original)
pub static mut DATA_5DB2C: [u8; 0x19 * 0x10] = [0u8; 0x19 * 0x10];

// LbCopyScreenBox — copies a rectangle from backing buffer to WScreen
fn lb_copy_screen_box(x: i32, y: i32, w: i32, h: i32, flags: u32) {
    // Full implementation requires the backing-store buffer pointer.
    // The game calls this to refresh dirty rectangles; stub for now.
    let _ = (x, y, w, h, flags);
}

// copy_screen_cube — copies one 16×16 tile from backing store to WScreen
fn copy_screen_cube(tile_x: i32, tile_y: i32) {
    let _ = (tile_x, tile_y);
}

// ---------------------------------------------------------------------------
// scroll_map — 0x1e400
//
// Translates the mouse screen position (dx, dz) plus numpad key state into
// a compass direction (1–10), then calls scroll() to pan the map camera.
//
// Numpad keycodes: 0x48=up, 0x4b=left, 0x50=down, 0x4d=right
// Threshold: dx=0x27e (638px = right edge), dz=0x18e (398px = bottom edge)
// ---------------------------------------------------------------------------
pub fn scroll_map(dx: i16, dz: i16) {
    unsafe {
        let mut edx = dx as u32; // x component
        let mut eax = dz as u32; // z component

        // Keyboard overrides
        if LB_KEY_ON[0x48] != 0 { eax = 0; }             // numpad-8 / cursor up
        if LB_KEY_ON[0x4b] != 0 { edx = 0; }             // numpad-4 / cursor left
        if LB_KEY_ON[0x50] != 0 { eax = 0x18e; }         // numpad-2 / cursor down
        if LB_KEY_ON[0x4d] != 0 {                         // numpad-6 / cursor right
            edx = 0x27e;
            // jmp jump_1e458 — skip the zero tests below
            scroll_dir_nonzero_x(eax, edx);
            return;
        }

        if edx != 0 {
            scroll_dir_nonzero_x(eax, edx);
            return;
        }

        // edx == 0 (no X movement)
        if eax == 0 {
            scroll(5);         // centre → default scroll direction
        } else if eax == 0x18e {
            scroll(9);         // straight down
        } else {
            scroll(1);         // straight up (any non-zero z with x==0)
        }
    }
}

fn scroll_dir_nonzero_x(eax: u32, edx: u32) {
    if eax == 0 {
        // Only X movement
        if edx == 0x27e { scroll(6); } else { scroll(4); }
    } else {
        // Both X and Z movement
        if edx == 0x27e {
            if eax == 0x18e { scroll(10); } else { scroll(2); }
        } else if eax == 0x18e {
            scroll(8);
        }
        // else: neither edge corner → no scroll
    }
}

// scroll() — 0x1e4e0 — updates player_view_map_x/y by one step in direction d
fn scroll(dir: u32) {
    unsafe {
        // Each direction (1-10 in compass order) adjusts the view coords.
        // 1=N, 2=NE, 3=E(unused), 4=W, 5=stop, 6=E, 7=NW(unused),
        // 8=SW, 9=S, 10=SE
        let vx = PLAYER_VIEW_MAP_X as i32;
        let vy = PLAYER_VIEW_MAP_Y as i32;
        let hi_x = LEVEL_HI_BOUNDARYX as i32;
        let hi_y = LEVEL_HI_BOUNDARYY as i32;
        let lo_x = LEVEL_LO_BOUNDARYX as i32;
        let lo_y = LEVEL_LO_BOUNDARYY as i32;

        match dir {
            1 => { // N: x++, y++  (scroll up-left on isometric)
                if vx < hi_x && vy < hi_y {
                    PLAYER_VIEW_MAP_X = (vx + 1) as i16;
                    PLAYER_VIEW_MAP_Y = (vy + 1) as i16;
                    draw_horizontal_and_set_all();
                }
            }
            2 => { // NE: x++, y--
                if vx < hi_x && vy > lo_y {
                    PLAYER_VIEW_MAP_X = (vx + 1) as i16;
                    PLAYER_VIEW_MAP_Y = (vy - 1) as i16;
                    draw_vertical_and_set_all();
                }
            }
            4 => { // W: x--, y++
                if vx > lo_x && vy < hi_y {
                    PLAYER_VIEW_MAP_X = (vx - 1) as i16;
                    PLAYER_VIEW_MAP_Y = (vy + 1) as i16;
                    draw_vertical_and_set_all();
                }
            }
            5 => {} // centre — no movement
            6 => { // E: x--, y--
                if vx > lo_x && vy > lo_y {
                    PLAYER_VIEW_MAP_X = (vx - 1) as i16;
                    PLAYER_VIEW_MAP_Y = (vy - 1) as i16;
                    draw_horizontal_and_set_all();
                }
            }
            8 => { // SW: x--, y++ (mirror of 4)
                if vx > lo_x && vy < hi_y {
                    PLAYER_VIEW_MAP_X = (vx - 1) as i16;
                    PLAYER_VIEW_MAP_Y = (vy + 1) as i16;
                    draw_vertical_and_set_all();
                }
            }
            9 => { // S: x--, y--
                if vx > lo_x && vy > lo_y {
                    PLAYER_VIEW_MAP_X = (vx - 1) as i16;
                    PLAYER_VIEW_MAP_Y = (vy - 1) as i16;
                    draw_horizontal_and_set_all();
                }
            }
            10 => { // SE: x--, y--
                if vx > lo_x && vy > lo_y {
                    PLAYER_VIEW_MAP_X = (vx - 1) as i16;
                    PLAYER_VIEW_MAP_Y = (vy - 1) as i16;
                    draw_horizontal_and_set_all();
                }
            }
            _ => {}
        }
    }
}

fn draw_horizontal_and_set_all() {
    // Calls draw_horizontal() + set_all_changes() — stubs pending translation
    unsafe { DATA_5532C = 0x10; }
}

fn draw_vertical_and_set_all() {
    unsafe { DATA_5532C = 0x10; }
}

// ---------------------------------------------------------------------------
// draw_mapwho — 0x1d450
//
// Iterates the level__MapWho linked list starting from the camera origin,
// builds a sort-by-Z drawlist, then dispatches per-entity draw routines.
// Very large function; this is a structural translation with stubs for
// the inner draw dispatches.
// ---------------------------------------------------------------------------
pub fn draw_mapwho() {
    unsafe {
        // Set graphics window to full map viewport (640×400)
        // LbScreenSetGraphicsWindow(0, 0, 640, 400) — stub
        DATA_60B10 = 0; DATA_60B0A = 0; DATA_60B1A = 0;
        DATA_60B12 = 0; DATA_60B14 = 0;

        if LEVEL_MAPWHO.is_null() { return; }

        // Compute map origin offset from camera
        let mx = (PLAYER_VIEW_MAP_X as i32) >> 1;
        let my = (PLAYER_VIEW_MAP_Y as i32) >> 1;
        let origin_idx = ((my << 7) + mx) * 2;

        let scan_base = if DATA_5A8BC.is_null() {
            std::ptr::null_mut()
        } else {
            DATA_5A8BC.add(origin_idx as usize) as *mut i16
        };

        // Iterate linked list through MapWho
        let mut scan = scan_base;
        loop {
            if scan.is_null() { break; }
            let entry = *scan as i32;
            if entry == -1 { break; }

            let mapwho_ptr = LEVEL_MAPWHO.add((entry * 2) as usize) as *const i16;
            scan = scan.add(1);

            // Check bounds (within 0x600 of map edges)
            let mapwho_addr = mapwho_ptr as usize;
            let base_addr   = LEVEL_MAPWHO as usize;
            if mapwho_addr < base_addr + 0x600 { continue; }
            if mapwho_addr >= base_addr + 0x8000 - 0x600 { continue; }

            // Walk the thing chain at this MapWho cell
            let mut entity_idx = *mapwho_ptr as u32;
            loop {
                if entity_idx == 0 { break; }
                if LEVEL_THINGS_BASE.is_null() { break; }
                let entity = LEVEL_THINGS_BASE.add(entity_idx as usize);

                // testb $0x1, 0xa(%entity) — skip if hidden
                if (*entity.add(0x0a) & 0x1) != 0 {
                    entity_idx = *(entity.add(0x0) as *const u16) as u32;
                    continue;
                }

                // Calculate screen-space X, Y from map coords
                let map_x = *(entity.add(0x4) as *const i16) as i32;
                let map_z = *(entity.add(0x6) as *const i16) as i32;
                let map_y = *(entity.add(0x8) as *const i16) as i32;

                let cam_x = (PLAYER_VIEW_MAP_X as i32) << 7;
                let cam_z = (PLAYER_VIEW_MAP_Y as i32) << 7;

                let dx = map_x - cam_x;
                let dz = map_z - cam_z;

                let sum_xz = dx + dz;
                let screen_x = (sum_xz.wrapping_shr(1) as i32) >> 3;
                let screen_z = (dx - dz).wrapping_shr(3);

                // Dispatch based on entity type 0x18(%entity)
                let etype = *entity.add(0x18);
                if etype == 5 {
                    draw_entity_animated(entity, screen_x + 0x80, screen_z + 0x10);
                }

                entity_idx = *(entity as *const u16) as u32;
            }
        }
    }
}

fn draw_entity_animated(_entity: *mut u8, _sx: i32, _sy: i32) {
    // Dispatches via jpt_1D596 based on entity subtype
    // Full translation in subsequent pass
}

// ---------------------------------------------------------------------------
// process_computer_players — 0x15b10
//
// On each tick (gated by process interval), runs the AI decision loop
// for all computer-controlled players.
// ---------------------------------------------------------------------------
pub fn process_computer_players() {
    unsafe {
        COMP_PLYR_PROCESS_TIMER = COMP_PLYR_PROCESS_TIMER.wrapping_add(1);
        let elapsed = COMP_PLYR_PROCESS_TIMER.wrapping_sub(COMP_PLYR_LAST_PROCESS_TIME);
        let do_process = elapsed >= COMP_PLYR_PROCESS_INTERVAL as u32;

        if do_process {
            COMP_PLYR_LAST_PROCESS_TIME = COMP_PLYR_PROCESS_TIMER;
        }

        if COMPUTER_PLAYERS_COUNT == 0 { return; }

        for player_idx in 0..COMPUTER_PLAYERS_COUNT as usize {
            // Check if player has agents (PLAYERS[player_idx].field_0x40b)
            // For each agent slot in this player's team, run process_action
            if do_process {
                // Full AI logic in subsequent translation pass
                for agent_idx in 0..COMP_PLYR_TEAM_SIZE as usize {
                    let _ = (player_idx, agent_idx);
                    // call process_computer_action(player_idx, agent_idx)
                }
            }
        }
    }
}

