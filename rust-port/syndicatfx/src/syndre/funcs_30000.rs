// syndre/funcs_30000.rs — Translated x86 functions at addresses 0x30000–0x3FFFF.
//
// This range contains: menu_select, level_finished, init_sound, etc.

use crate::globals::*;

pub fn menu_select_impl() -> u8 {
    // 0x37600 — presents the mission briefing/selection menu.
    // Returns non-zero when a mission has been selected.
    // Full translation: reads gui_strings, handles UI input loop.
    // Stub: return 1 (immediately start first mission) for testing.
    1
}
