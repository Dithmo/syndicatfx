// syndre/data.rs — Static data tables and globals from syndre.sx.
//
// These correspond to the GLOBAL() and .data/.rodata sections of the original.
// All are unsafe statics matching the original memory layout.

use crate::game_data::CPObjective;

// ---- Music state ----------------------------------------------------------
pub static mut DATA_5C354: i32 = 0;  // current music track request
pub static mut DATA_5C34C: i32 = 0;  // previous music state

// ---- Map / tile data pointers ---------------------------------------------
pub static mut DATA_55358: *mut u8 = std::ptr::null_mut();  // map block ptr

// ---- Misc data tables (addresses from syndre.sx) --------------------------
pub static mut DATA_5532C: u16 = 0;  // unknown u16 flag
pub static mut DATA_5A510: [u8; 256] = [0u8; 256]; // collision type table
pub static mut DATA_5E551: [u8; 512] = [0u8; 512]; // player slot data
pub static mut DATA_5E552: [i8; 512] = [0i8; 512]; // player slot data (signed)

// ---- Level data (from GLOBAL exports in syndre.sx) -----------------------
pub static mut LEVEL_PEOPLE: *mut u8 = std::ptr::null_mut();  // level__People base
pub static mut LEVEL_THINGS_BASE: *mut u8 = std::ptr::null_mut(); // level__things_base

// Horizontal distance table (used by getrdist)
pub static mut GETRDIST_TABLE: [u16; 512] = [0u16; 512];

// ---- Load file arrays (from syndre.sx .data section) --------------------
pub static mut SOUND_BANK_FILES0: *mut u8 = std::ptr::null_mut();
pub static mut UNK1_EMPTY_LOAD_FILES: *mut u8 = std::ptr::null_mut();
pub static mut UNK984_LOAD_FILES: *mut u8 = std::ptr::null_mut();
