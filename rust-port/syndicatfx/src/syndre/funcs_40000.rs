// syndre/funcs_40000.rs — Translated x86 functions at addresses 0x40000+.

use crate::syndre::data::DATA_5A95E;

// ---------------------------------------------------------------------------
// 0x29460  get_angle
//
// Thin wrapper: sign-extends both i16 args then delegates to arctan.
// ---------------------------------------------------------------------------
pub fn get_angle(dx: i32, dy: i32) -> u8 {
    arctan(dx as i16, dy as i16) as u8
}

// ---------------------------------------------------------------------------
// 0x4fad9  arctan
//
// 8-direction fixed-point arctangent. Maps a 2-D delta (dx, dy) to an angle
// in the range 0-255 (one full turn = 256 units, 0 = pointing south (+Y),
// 64 = east (+X), 128 = north (-Y), 192 = west (-X)).
//
// Algorithm: normalise into octant, compute ratio = min(|dx|,|dy|)/max * 256,
// look up DATA_5A95E[ratio] (0-32 = 0-45°), then add octant offset.
//
// Literal translation: control-flow mirrors the eight jump targets.
// The x86 `div` is emulated as (abs * 256 / max) truncated to u8.
// ---------------------------------------------------------------------------
#[inline]
fn arctan_ratio(min: u16, max: u16) -> usize {
    if max == 0 { return 0; }
    ((min as u32).wrapping_mul(256) / max as u32) as u8 as usize
}

pub fn arctan(dx: i16, dy: i16) -> u16 {
    // mirror the initial zero checks
    let ax: i16;
    let bx_neg: i16; // holds -dy (what the assembly calls bx after `neg %bx`)
    if dx == 0 {
        if dy == 0 { return 0; }
        ax = 0;
        bx_neg = dy.wrapping_neg();
    } else {
        ax = dx;
        bx_neg = dy.wrapping_neg();
    }
    // From here: ax = dx, bx_neg = -dy
    if ax >= 0 {
        // --- dx ≥ 0 ---
        if bx_neg >= 0 {
            // -dy ≥ 0 → dy ≤ 0; ax=dx, bx_neg=|dy|
            if ax >= bx_neg {
                // swap: ratio = |dy| / dx
                let r = arctan_ratio(bx_neg as u16, ax as u16);
                (DATA_5A95E[r] as u16).wrapping_add(0x40)
            } else {
                // ratio = dx / |dy|; neg+0x80
                let r = arctan_ratio(ax as u16, bx_neg as u16);
                (DATA_5A95E[r] as i16).wrapping_neg() as u16 & 0xffff_u16
                    | 0x80_u16  // effectively 0x80 - table[r] masked to 8 bits
                // matches: neg %ax; add $0x80,%ax
            }
        } else {
            // -dy < 0 → dy > 0; bx = |dy| = -bx_neg
            let bx = bx_neg.wrapping_neg(); // = dy > 0
            if ax >= bx {
                // swap
                let r = arctan_ratio(bx as u16, ax as u16);
                let v = (DATA_5A95E[r] as i16).wrapping_neg();
                (v.wrapping_add(0x40)) as u16 & 0xff
            } else {
                let r = arctan_ratio(ax as u16, bx as u16);
                DATA_5A95E[r] as u16 & 0xff
            }
        }
    } else {
        // --- dx < 0; ax_abs = |dx| ---
        let ax_abs = ax.wrapping_neg();
        if bx_neg >= 0 {
            // -dy ≥ 0 → dy ≤ 0
            if ax_abs >= bx_neg {
                // swap
                let r = arctan_ratio(bx_neg as u16, ax_abs as u16);
                let v = (DATA_5A95E[r] as i16).wrapping_neg();
                (v.wrapping_add(0xc0_i16)) as u16 & 0xff
            } else {
                let r = arctan_ratio(ax_abs as u16, bx_neg as u16);
                (DATA_5A95E[r] as u16).wrapping_add(0x80) & 0xff
            }
        } else {
            // -dy < 0 → dy > 0
            let bx = bx_neg.wrapping_neg();
            if ax_abs >= bx {
                // swap
                let r = arctan_ratio(bx as u16, ax_abs as u16);
                (DATA_5A95E[r] as u16).wrapping_add(0xc0) & 0xff
            } else {
                let r = arctan_ratio(ax_abs as u16, bx as u16);
                let v = (DATA_5A95E[r] as i16).wrapping_neg();
                (v.wrapping_add(0x100_i16)) as u16 & 0xff
            }
        }
    }
}
