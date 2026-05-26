// main.rs — mirrors main.c: argument parsing, init, and top-level game loop.

mod game;
mod game_data;
mod globals;
mod display;
mod keyboard;
mod mouse;
mod sound;
mod dos;
mod util;
mod timer;
mod syndre;
mod osunix;

use globals::*;
use game::{game_initialise, game_quit, syndicate};
use display::{display_create_vga_buffer, display_free_vga_buffer,
              display_set_full_screen, display_set_lowres_stretch};
use sound::{audio_options_set_default, init_audio};
use keyboard::init_buffered_keys;
use game_data::setup_file_names;
use bflibrary::memory::LbMemoryReset;
use bflibrary::keyboard::{LbIKeyboardOpen, LB_MOUSE_AUTO_RESET};
use bflibrary::mouse::LbMouseSetup;
use bflibrary::data::{LbDataLoadAll, LbDataFreeAll};
use bflibrary::timer::{LbTimerClock, LbSleepUntil};

static LANGS: &[&str] = &["eng", "fre", "ita", "ger", "pol", "spa"];

fn print_banner() {
    println!("SyndicatFX ver 0.0.6");
    println!("The original by Bullfrog. Port solution by Unavowed and Gynvael Coldwind.");
    println!("Refactored port base by Mefistotelis.");
    println!("Expanded by other fans, signed in commits.");
}

fn print_help(prog: &str) {
    println!("Usage: {} [OPTIONS]\n", prog);
    println!("  --windowed    -w    Run in windowed mode");
    println!("  --no-stretch  -S    Don't stretch 320x200 to 640x480");
    println!("  --help        -h    Display this message");
}

fn process_options(args: &[String]) {
    unsafe {
        let mut i = 1usize;
        while i < args.len() {
            match args[i].as_str() {
                "--windowed" | "-w" => { CMDLN_FULLSCREEN = false; }
                "--no-stretch" | "-S" => { CMDLN_LORES_STRETCH = false; }
                "--nosound" | "-s" => {
                    use bfsoundlib::audio::{AUDIO_ABLE_MUSIC, AUDIO_ABLE_SOUND};
                    AUD_FLAGS &= !(AUDIO_ABLE_MUSIC | AUDIO_ABLE_SOUND);
                }
                "--help" | "-h" => {
                    print_help(&args[0]);
                    std::process::exit(0);
                }
                "--nport" | "-p" => {
                    i += 1;
                    if let Some(v) = args.get(i) {
                        UNUSED_OPTION_P = v.parse::<u8>().unwrap_or(0).min(16);
                    }
                }
                "--nname" | "-n" => {
                    i += 1;
                    if let Some(v) = args.get(i) {
                        let b = v.as_bytes();
                        let len = b.len().min(NETWORK_NAME.len() - 1);
                        NETWORK_NAME[..len].copy_from_slice(&b[..len]);
                        NETWORK_NAME[len] = 0;
                    }
                }
                "--colang" | "-c" => {
                    i += 1;
                    if let Some(v) = args.get(i) {
                        let lang = v.parse::<u8>().unwrap_or(0);
                        LANGUAGE = if lang >= 6 { 0 } else { lang };
                    }
                }
                "--lores" | "-L" => { DRAW_FLAGS = DRW_F_UNKN04; }
                "--hires" | "-H" => { DRAW_FLAGS = DRW_F_SCREEN_VRES16; }
                "--iirq" | "-R" => {
                    i += 1;
                    if let Some(v) = args.get(i) {
                        SNDCARD_IRQ = v.parse().unwrap_or(0);
                    }
                }
                "--idma" | "-D" => {
                    i += 1;
                    if let Some(v) = args.get(i) {
                        SNDCARD_DMA = v.parse().unwrap_or(0);
                    }
                }
                "--iio" | "-I" => {
                    i += 1;
                    if let Some(v) = args.get(i) {
                        SNDCARD_IOADDR = v.parse().unwrap_or(0);
                    }
                }
                _ => {}
            }
            i += 1;
        }

        // Set 3-char language string
        let lang_str = LANGS.get(LANGUAGE as usize).copied().unwrap_or("eng");
        let b = lang_str.as_bytes();
        LANGUAGE_3STR[..3].copy_from_slice(b);
        LANGUAGE_3STR[3] = 0;
        // Restrict language to 0..2 for the multilingual CD
        if LANGUAGE >= 3 { LANGUAGE = 0; }
    }
}

// Additional globals only needed in main
static mut CMDLN_FULLSCREEN:    bool = true;
static mut CMDLN_LORES_STRETCH: bool = true;
static mut AUD_FLAGS: u16 = bfsoundlib::audio::AUDIO_ABLE_MUSIC
                           | bfsoundlib::audio::AUDIO_ABLE_SOUND;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    print_banner();

    unsafe {
        audio_options_set_default();

        // Default state (mirrors the initialisation block in main.c)
        CURRENT_LEVNO    = 1;
        LEVEL_MAP_NUMBER = 1;
        DRAW_FLAGS       = DRW_F_SCREEN_VRES16;
        IS_MULTIPLAYER_GAME = 0;
        BYTE_60B42       = 0;
        BYTE_60B44       = 0;
        BYTE_60B3B       = 0;
        BYTE_60B3A       = 1;
        BYTE_60B4C       = 0;
        BYTE_60B47       = 1;
        CHEAT_CREDITS    = 0;
        CHEAT_WORLDMAP   = 0;
        DEBUG_K          = 0;
        BYTE_60B51       = 0;
        LANGUAGE         = 0;
        UNUSED_OPTION_P  = 0;
        CHEATS_SPEEDUP   = 0;
        CHEATS_MISSION   = 0;

        process_options(&args);

        if !game_initialise() {
            std::process::exit(1);
        }

        display_create_vga_buffer();
        display_set_full_screen(CMDLN_FULLSCREEN);
        display_set_lowres_stretch(CMDLN_LORES_STRETCH);

        if (DRAW_FLAGS & DRW_F_SCREEN_VRES16) != 0 {
            // LbDataLoadAll(LOAD_FILES_VRES16); — load game assets
        } else if (DRAW_FLAGS & DRW_F_UNKN04) != 0 {
            BYTE_60B42 = 0;
            // LbDataLoadAll(LOAD_FILES_MCGA);
        }

        syndre::funcs_10000::ApSpriteSetup_ForceHeight(
            POINTER_SPRITES as *mut u8,
            POINTER_SPRITES_END as *mut u8,
            POINTER_DATA);
        MOUSE_SWAP = 1;

        // read_gui_strings_file(); — stub
        // create_gui_strings_list(); — stub

        LB_MOUSE_AUTO_RESET = false;
        MOUSE_SPRITE = if POINTER_SPRITES.is_null() {
            std::ptr::null_mut()
        } else {
            POINTER_SPRITES.add(1)
        };

        LbMouseSetup(MOUSE_SPRITE as *const _, 256, 256);
        LbIKeyboardOpen();

        init_audio();

        if BYTE_60B42 == 0 {
            init_buffered_keys();
            syndicate();
            // After syndicate() returns (shouldn't normally), sleep briefly
            let end = LbTimerClock() + 72 * 5000 / 91;
            LbSleepUntil(end);
        }

        // LbDataFreeAll(LOAD_FILES_VRES16);

        display_free_vga_buffer();
        LbMemoryReset();
        game_quit();
    }
}
