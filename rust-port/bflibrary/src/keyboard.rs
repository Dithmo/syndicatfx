// LbKeyboard* — SDL2-backed keyboard state, matching bfkeybd.h interface.

use crate::types::*;

// Key code constants (KC_*) — match bfkeybd.h
pub const KC_F1:  u8 = 59;
pub const KC_F2:  u8 = 60;
pub const KC_F3:  u8 = 61;
pub const KC_F4:  u8 = 62;
pub const KC_F5:  u8 = 63;
pub const KC_F6:  u8 = 64;
pub const KC_F7:  u8 = 65;
pub const KC_F8:  u8 = 66;
pub const KC_F9:  u8 = 67;
pub const KC_F10: u8 = 68;
pub const KC_ESC: u8 = 1;
pub const KC_RETURN: u8 = 28;
pub const KC_SPACE:  u8 = 57;
pub const KC_BACK:   u8 = 14;
pub const KC_TAB:    u8 = 15;

// Modifier flags (KMod_*)
pub const KMOD_SHIFT:   u32 = 0x03;
pub const KMOD_CONTROL: u32 = 0x0C;
pub const KMOD_ALT:     u32 = 0x30;

// Global keyboard state (mirrors lbKeyOn[], lbInkey, lbShift)
pub static mut LB_KEY_ON: [u8; 256] = [0u8; 256];
pub static mut LB_INKEY: u8 = 0;
pub static mut LB_SHIFT: u32 = 0;
pub static mut LB_INKEY_PREFIXED: bool = false;

// ASCII lookup tables (standard PC scancode → ASCII, simplified)
pub static mut LB_INKEY_TO_ASCII: [u8; 128] = {
    let mut t = [0u8; 128];
    // Basic printable chars at common scan codes
    t[2]  = b'1'; t[3]  = b'2'; t[4]  = b'3'; t[5]  = b'4'; t[6]  = b'5';
    t[7]  = b'6'; t[8]  = b'7'; t[9]  = b'8'; t[10] = b'9'; t[11] = b'0';
    t[16] = b'q'; t[17] = b'w'; t[18] = b'e'; t[19] = b'r'; t[20] = b't';
    t[21] = b'y'; t[22] = b'u'; t[23] = b'i'; t[24] = b'o'; t[25] = b'p';
    t[30] = b'a'; t[31] = b's'; t[32] = b'd'; t[33] = b'f'; t[34] = b'g';
    t[35] = b'h'; t[36] = b'j'; t[37] = b'k'; t[38] = b'l';
    t[44] = b'z'; t[45] = b'x'; t[46] = b'c'; t[47] = b'v'; t[48] = b'b';
    t[49] = b'n'; t[50] = b'm';
    t[57] = b' '; t[28] = b'\r';
    t
};

pub static mut LB_INKEY_TO_ASCII_SHIFT: [u8; 128] = {
    let mut t = [0u8; 128];
    t[2]  = b'!'; t[3]  = b'@'; t[4]  = b'#'; t[5]  = b'$'; t[6]  = b'%';
    t[7]  = b'^'; t[8]  = b'&'; t[9]  = b'*'; t[10] = b'('; t[11] = b')';
    t[16] = b'Q'; t[17] = b'W'; t[18] = b'E'; t[19] = b'R'; t[20] = b'T';
    t[21] = b'Y'; t[22] = b'U'; t[23] = b'I'; t[24] = b'O'; t[25] = b'P';
    t[30] = b'A'; t[31] = b'S'; t[32] = b'D'; t[33] = b'F'; t[34] = b'G';
    t[35] = b'H'; t[36] = b'J'; t[37] = b'K'; t[38] = b'L';
    t[44] = b'Z'; t[45] = b'X'; t[46] = b'C'; t[47] = b'V'; t[48] = b'B';
    t[49] = b'N'; t[50] = b'M';
    t[57] = b' '; t[28] = b'\r';
    t
};

pub static mut LB_KEYBOARD_HANDLER: Option<fn(u32, u8) -> TbResult> = None;

// Key action constants
pub const K_ACTN_KEYDOWN: u32 = 1;
pub const K_ACTN_KEYUP:   u32 = 0;

pub static mut LB_MOUSE_AUTO_RESET: bool = false;

pub fn LbKeyboardOpen() -> TbResult {
    // SDL keyboard is always open once SDL is initialized.
    LB_SUCCESS
}

pub fn LbIKeyboardOpen() -> TbResult {
    LbKeyboardOpen()
}

pub fn LbKeyboardClose() -> TbResult {
    unsafe {
        LB_INKEY = 0;
        LB_SHIFT = 0;
        LB_KEY_ON = [0u8; 256];
        LB_KEYBOARD_HANDLER = None;
    }
    LB_SUCCESS
}

pub fn LbKeyboardCustomHandler(f: fn(u32, u8) -> TbResult) {
    unsafe { LB_KEYBOARD_HANDLER = Some(f); }
}

// Called by the SDL event loop to update key state.
pub fn lb_keyboard_update(action: u32, scancode: u8) {
    unsafe {
        if (scancode as usize) < LB_KEY_ON.len() {
            LB_KEY_ON[scancode as usize] = if action == K_ACTN_KEYDOWN { 1 } else { 0 };
        }
        if action == K_ACTN_KEYDOWN {
            LB_INKEY = scancode;
        } else {
            LB_INKEY = 0;
        }
        if let Some(handler) = LB_KEYBOARD_HANDLER {
            handler(action, scancode);
        }
    }
}
