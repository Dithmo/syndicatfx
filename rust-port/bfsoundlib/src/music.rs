// MIDI / music sequencing — mirrors bfmusic.h / InitMIDI / BFMidi*.

use crate::audio::{GetMusicAble, GetMusicActive};

pub static mut SONG_CURRENTLY_PLAYING: u16 = 0;

pub fn InitMIDI(bank_fname: *const libc::c_char, drv_fname: *const libc::c_char,
                irq: u16, dma: u16, ioaddr: u16) -> i32 {
    let _ = (bank_fname, drv_fname, irq, dma, ioaddr);
    // WildMIDI integration goes here; stub returns success.
    1
}

pub fn ShutdownMIDI() {
    BFMidiStopMusic();
}

pub fn BFMidiStartMusic(song_no: u16) {
    if !GetMusicAble() || !GetMusicActive() { return; }
    unsafe {
        if SONG_CURRENTLY_PLAYING != song_no {
            SONG_CURRENTLY_PLAYING = song_no;
            // Dispatch to WildMIDI / AIL here
        }
    }
}

pub fn BFMidiStopMusic() {
    unsafe { SONG_CURRENTLY_PLAYING = 0; }
}

pub fn BFMidiPauseSong() {}
pub fn BFMidiResumeSong() {}

pub fn BFMidiIsMusicPlaying() -> bool {
    unsafe { SONG_CURRENTLY_PLAYING > 0 }
}

pub fn FreeMusic() {
    ShutdownMIDI();
}
