// All shared mutable global state that was previously split between C globals
// and EXPORT_SYMBOL() references in syndre.sx.
//
// In Rust these are static mut — accessed via unsafe blocks everywhere, which
// mirrors the original C semantics exactly (no locking, single-threaded game).

use bflibrary::TbSprite;
use crate::sound::BFSample;
use crate::sound::BFSampleStatus;
use crate::game_data::CPObjective;
use crate::game_data::TbLoadFiles;

// ---- Game control flags (game.h) ------------------------------------------

pub static mut BYTE_60AFC: u8 = 0;          // game state/control flags
pub static mut IS_MULTIPLAYER_GAME: u8 = 0;

// ---- Display flags (display.h) --------------------------------------------

pub static mut DRAW_FLAGS: u8 = 0;
pub const DRW_F_SCREEN_MCGA:   u8 = 0x1;
pub const DRW_F_SCREEN_VRES16: u8 = 0x2;
pub const DRW_F_UNKN04:        u8 = 0x4;

pub static mut VGA_BUFFER:      *mut u8 = std::ptr::null_mut();
pub static mut WSCREEN:         *mut u8 = std::ptr::null_mut();
pub static mut VSCREEN:         *mut u8 = std::ptr::null_mut();
pub static mut USCREEN:         *mut u8 = std::ptr::null_mut();
pub static mut BSCREEN:         *mut u8 = std::ptr::null_mut();
pub static mut GRAPHICS_PALETTE:*mut u8 = std::ptr::null_mut();

// Scaled mouse coordinates exposed to ASM game logic
pub static mut LB_DISPLAY_MOUSE_X_640:  i16 = 0;
pub static mut LB_DISPLAY_MOUSE_Y_400:  i16 = 0;
pub static mut LB_DISPLAY_MMOUSE_X_640: i16 = 0;
pub static mut LB_DISPLAY_MMOUSE_Y_400: i16 = 0;

// ---- Sound card settings (sound.h) ----------------------------------------

pub static mut SNDCARD_IRQ:    u16 = 0;
pub static mut SNDCARD_DMA:    u16 = 0;
pub static mut SNDCARD_IOADDR: u16 = 0;

// ---- Sample tables (from syndre.sx GLOBAL exports) ------------------------

pub static mut SMPTABLE:     *mut BFSample = std::ptr::null_mut();
pub static mut SMPTABLE_END: *mut BFSample = std::ptr::null_mut();
pub static mut SMPDATA:      *mut u8       = std::ptr::null_mut();
pub static mut SAMPLE_STATUS: [BFSampleStatus; 256] = [BFSampleStatus { field_0: 0, field_1: 0 }; 256];

pub static mut BYTE_5BBE8: u8 = 0;
pub static mut BYTE_5BBE9: u8 = 0;

// ---- Level / game data (game_data.h + syndre.sx exports) ------------------

pub static mut LEVEL_SEED: u16 = 0;
pub static mut LEVEL_CPOBJECTIVES: [CPObjective; 128] = [CPObjective::zero(); 128];

pub static mut CURRENT_LEVNO:    u16 = 0;
pub static mut LEVEL_MAP_NUMBER: u16 = 0;

// ---- Sprite / graphics resources ------------------------------------------

pub static mut POINTER_SPRITES:     *mut TbSprite = std::ptr::null_mut();
pub static mut POINTER_SPRITES_END: *mut TbSprite = std::ptr::null_mut();
pub static mut POINTER_DATA:        *mut u8       = std::ptr::null_mut();
pub static mut MOUSE_SPRITE:        *mut TbSprite = std::ptr::null_mut();

pub static mut M_SPRITES:     *mut TbSprite = std::ptr::null_mut();
pub static mut M_SPRITES_END: *mut TbSprite = std::ptr::null_mut();
pub static mut M_FONT:        *mut TbSprite = std::ptr::null_mut();
pub static mut M_FONT_END:    *mut TbSprite = std::ptr::null_mut();
pub static mut H_SPRITES:     *mut TbSprite = std::ptr::null_mut();
pub static mut H_SPRITES_END: *mut TbSprite = std::ptr::null_mut();

pub static mut M_LOGOS:        *mut u8 = std::ptr::null_mut();
pub static mut H_COL:          *mut u8 = std::ptr::null_mut();
pub static mut MAP_BUF:        *mut u8 = std::ptr::null_mut();
pub static mut M_SELECT_PAL:   *mut u8 = std::ptr::null_mut();
pub static mut H_FONT:         *mut u8 = std::ptr::null_mut();
pub static mut H_BLOCKS:       *mut u8 = std::ptr::null_mut();
pub static mut H_SPRITES_DATA: *mut u8 = std::ptr::null_mut();
pub static mut FRAMES:         *mut u8 = std::ptr::null_mut();
pub static mut FRAMES_END:     *mut u8 = std::ptr::null_mut();
pub static mut ELEMENTS_ANI:     *mut u8 = std::ptr::null_mut();
pub static mut ELEMENTS_ANI_END: *mut u8 = std::ptr::null_mut();
pub static mut STARTS_ANI:       *mut u8 = std::ptr::null_mut();

pub static mut LOAD_FILES_VRES16: *mut TbLoadFiles = std::ptr::null_mut();
pub static mut LOAD_FILES_MCGA:   *mut TbLoadFiles = std::ptr::null_mut();

// ---- Network / multiplayer ------------------------------------------------

pub static mut NETWORK_SLOT: i16    = 0;
pub static mut NETWORK_NAME: [u8; 18] = [0u8; 18];

// ---- Mouse state ----------------------------------------------------------

pub static mut MOUSE_SWAP: u32 = 0;
pub static mut MOUSE_OLD_W: u16 = 0;

// ---- Misc game state (from syndre.sx) ------------------------------------

pub static mut LANGUAGE: u8 = 0;
pub static mut LANGUAGE_3STR: [u8; 4] = [b'e', b'n', b'g', 0];

pub static mut BYTE_60B42: u8 = 0;
pub static mut BYTE_60B44: u8 = 0;
pub static mut BYTE_60B3B: u8 = 0;
pub static mut BYTE_60B3A: u8 = 0;
pub static mut BYTE_60B4C: u8 = 0;
pub static mut BYTE_60B47: u8 = 0;
pub static mut BYTE_60B51: u8 = 0;

pub static mut CHEAT_CREDITS:  u8 = 0;
pub static mut CHEAT_WORLDMAP: u8 = 0;
pub static mut CHEATS_SPEEDUP: u8 = 0;
pub static mut CHEATS_MISSION: u8 = 0;
pub static mut DEBUG_K: i32 = 0;
pub static mut UNUSED_OPTION_P: u8 = 0;

// ---- Cheat / world map state (syndre.sx) ----------------------------------

pub static mut COUNTRY_STATES: [u8; 512] = [0u8; 512]; // sized from syndre
// DATA_5C354 lives in syndre::data to avoid duplicate definition

// ---- Keyboard (forwarded from bflibrary for ASM access) ------------------
// These are re-exported here so game/syndre code can import from one place.
pub use bflibrary::keyboard::{LB_KEY_ON  as LB_KEY_ON_ARR,
                               LB_INKEY   as LB_INKEY_VAL,
                               LB_SHIFT   as LB_SHIFT_VAL,
                               LB_INKEY_PREFIXED,
                               LB_INKEY_TO_ASCII,
                               LB_INKEY_TO_ASCII_SHIFT,
                               KC_F10};
