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
use crate::sound::{BFMidiStartMusic, BFMidiIsMusicPlaying, SetBFSampleStatus};
use bflibrary::keyboard::LB_KEY_ON;

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

// ---------------------------------------------------------------------------
// set_mission_complete — forward declaration (defined in funcs_20000.rs)
// ---------------------------------------------------------------------------
pub fn set_mission_complete() {
    crate::syndre::funcs_20000::set_mission_complete_impl();
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

pub fn set_default_player() {
    crate::syndre::funcs_20000::set_default_player_impl();
}

pub fn menu_select() -> u8 {
    crate::syndre::funcs_30000::menu_select_impl()
}

pub fn reset_mission_info() {
    crate::syndre::funcs_20000::reset_mission_info_impl();
}

pub fn initialise_player() {
    crate::syndre::funcs_20000::initialise_player_impl();
}

pub fn process_players_turn() {
    crate::syndre::funcs_20000::process_players_turn_impl();
}

pub fn multi_play() {
    crate::syndre::funcs_20000::multi_play_impl();
}

pub fn single_play() {
    crate::syndre::funcs_20000::single_play_impl();
}

pub fn process_computer_players() {
    // 0x15b10 — AI logic; full translation in next pass
}

pub fn move_it() {
    crate::syndre::funcs_20000::move_it_impl();
}

pub fn correct_buttons() -> u8 {
    crate::syndre::funcs_20000::correct_buttons_impl()
}

pub fn process_day(_ticks: u32) -> u8 {
    // 0x18810 — stub
    0
}

pub fn check_end_level() {
    unsafe {
        // 0x105e0 — check win/lose condition; stub sets byte_60AFC bit
        // Full translation: checks mission objectives via level__CPObjectives
    }
}

pub fn swap_screen_vres16() {
    // Calls back into C display layer
    crate::display::swap_wscreen();
}

pub fn copy_back() {
    // 0x1df10 — copies backbuffer region; stub
}

pub fn draw_mapwho() {
    // 0x1d450 — renders map entities; stub
}

pub fn draw_panel() {
    // 0x2d0e0 — renders UI panel; stub
}

pub fn scroll_map(dx: i16, dz: i16) {
    // 0x1e400 — scrolls camera; stub
    let _ = (dx, dz);
}

pub fn click_map() {
    // 0x11ea0 — processes mouse clicks on map; stub
}

pub fn level_finished() {
    // 0x37f80 — handles post-mission sequence; stub
}

pub fn transfer_people_into_player(slot: i16) {
    // 0x246d0 — transfers agent units; stub
    let _ = slot;
}

pub fn ASM_free_map_level() {
    // 0x24fe0 — frees level memory; stub
}

pub fn ApSpriteSetup_ForceHeight(start: *mut bflibrary::TbSprite,
                                   end:   *mut bflibrary::TbSprite,
                                   data:  *mut u8) -> *mut i32 {
    // 0x27c60 — adjusts sprite height tables; stub
    let _ = (start, end, data);
    std::ptr::null_mut()
}
