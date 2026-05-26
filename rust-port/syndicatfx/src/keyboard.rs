// keyboard.rs — mirrors keyboard.c: 16-entry circular key buffer.

use bflibrary::keyboard::{
    LbKeyboardCustomHandler, LB_KEY_ON, LB_INKEY, LB_SHIFT,
    LB_INKEY_PREFIXED, LB_INKEY_TO_ASCII, LB_INKEY_TO_ASCII_SHIFT,
    K_ACTN_KEYDOWN, KMOD_SHIFT,
};
use bflibrary::types::TbResult;

const KEYBOARD_BUFFER_SIZE: usize = 16;

pub static mut BUFFERED_KEYS:            [u32; KEYBOARD_BUFFER_SIZE] = [0u32; 16];
pub static mut BUFFERED_KEYS_READ_INDEX: u32 = 0;
pub static mut BUFFERED_KEYS_WRITE_INDEX:u32 = 0;

fn add_key_to_buffer(key: u8) {
    unsafe {
        BUFFERED_KEYS[BUFFERED_KEYS_WRITE_INDEX as usize] = key as u32;
        let new_write = (BUFFERED_KEYS_WRITE_INDEX + 1) % KEYBOARD_BUFFER_SIZE as u32;
        if new_write != BUFFERED_KEYS_READ_INDEX {
            BUFFERED_KEYS_WRITE_INDEX = new_write;
        }
    }
}

pub fn next_buffered_key() -> u32 {
    unsafe {
        if BUFFERED_KEYS_READ_INDEX == BUFFERED_KEYS_WRITE_INDEX { return 0; }
        let key = BUFFERED_KEYS[BUFFERED_KEYS_READ_INDEX as usize];
        BUFFERED_KEYS_READ_INDEX = (BUFFERED_KEYS_READ_INDEX + 1) % KEYBOARD_BUFFER_SIZE as u32;
        key
    }
}

pub fn read_buffered_char() -> char {
    unsafe {
        let key = next_buffered_key() as usize;
        if key >= 128 { return '\0'; }
        let ascii = if (LB_SHIFT & KMOD_SHIFT) != 0 {
            LB_INKEY_TO_ASCII_SHIFT[key]
        } else {
            LB_INKEY_TO_ASCII[key]
        };
        ascii as char
    }
}

pub fn read_char() -> char {
    unsafe {
        let key = LB_INKEY as usize;
        if key >= 128 { return '\0'; }
        let ascii = if (LB_SHIFT & KMOD_SHIFT) != 0 {
            LB_INKEY_TO_ASCII_SHIFT[key]
        } else {
            LB_INKEY_TO_ASCII[key]
        };
        ascii as char
    }
}

pub fn reset_buffered_keys() {
    unsafe {
        BUFFERED_KEYS_READ_INDEX  = 0;
        BUFFERED_KEYS_WRITE_INDEX = 0;
    }
}

fn k_event_buffered_keys_update(action: u32, code: u8) -> TbResult {
    unsafe {
        if action == K_ACTN_KEYDOWN && !LB_INKEY_PREFIXED {
            add_key_to_buffer(code);
        }
    }
    bflibrary::types::LB_OK
}

pub fn init_buffered_keys() {
    LbKeyboardCustomHandler(k_event_buffered_keys_update);
}
