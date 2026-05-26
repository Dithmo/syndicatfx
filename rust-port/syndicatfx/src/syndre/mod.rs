// syndre/mod.rs — The translated x86 assembly core of Syndicate.
//
// Strategy: literal translation.
//   • Each register becomes a local u32 variable with its name preserved.
//   • Complex control flow uses a `pc` (program counter) state-machine loop
//     so any jump target can be reached without `goto`.
//   • Arithmetic uses wrapping_* to match 32-bit unsigned overflow.
//   • Signed division (idiv) uses i64 to preserve the 64-bit dividend.
//   • Global state is accessed via the `globals` module.

pub mod funcs_10000;
pub mod funcs_20000;
pub mod funcs_30000;
pub mod funcs_40000;
pub mod data;

pub use funcs_10000::*;
pub use funcs_20000::*;
pub use funcs_30000::*;
pub use funcs_40000::*;
pub use data::*;

// Re-export the entry points that game.rs calls directly.
// Most of these will be stubs until the corresponding block in
// funcs_*.rs is fully translated.

extern "C" {
    // Functions not yet translated remain as unresolved symbols that will
    // be linked from the remaining C/ASM until fully ported.
}

// Convenience: signed arithmetic helpers matching x86 idiv semantics.
#[inline(always)]
pub(crate) fn idiv32(hi: u32, lo: u32, divisor: u32) -> (u32, u32) {
    // x86 idiv: signed divide edx:eax by divisor → eax=quotient, edx=remainder
    let dividend = (((hi as i64) << 32) | (lo as i64));
    let d = divisor as i32 as i64;
    if d == 0 { panic!("divide by zero"); }
    let q = dividend / d;
    let r = dividend % d;
    (q as u32, r as u32)
}

// SAR (arithmetic right shift) helper
#[inline(always)]
pub(crate) fn sar(val: u32, count: u32) -> u32 {
    ((val as i32) >> count) as u32
}
