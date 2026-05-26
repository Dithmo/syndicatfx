// Audio initialisation — mirrors bfaudio.h / InitAudio / FreeAudio.

pub static mut SOUND_ABLE:  bool = false;
pub static mut MUSIC_ABLE:  bool = false;
pub static mut SOUND_ACTIVE: bool = false;
pub static mut MUSIC_ACTIVE: bool = false;

#[repr(C)]
pub struct AudioInitOptions {
    pub sound_data_path:            *const libc::c_char,
    pub sound_driver_path:          *const libc::c_char,
    pub ini_path:                   *const libc::c_char,
    pub auto_scan:                  u8,
    pub stereo_option:              u8,
    pub disable_load_sounds:        u8,
    pub disable_load_music:         u8,
    pub init_redbook_audio:         u8,
    pub use_current_awe32_soundfont:u8,
    pub able_flags:                 u16,
    pub sound_type:                 u32,
    pub max_samples:                u16,
}

pub const AUDIO_ABLE_MUSIC: u16 = 0x01;
pub const AUDIO_ABLE_SOUND: u16 = 0x02;

pub fn InitAudio(opts: *const AudioInitOptions) -> i32 {
    unsafe {
        if opts.is_null() { return 0; }
        let o = &*opts;
        SOUND_ABLE  = (o.able_flags & AUDIO_ABLE_SOUND) != 0;
        MUSIC_ABLE  = (o.able_flags & AUDIO_ABLE_MUSIC) != 0;
        SOUND_ACTIVE = SOUND_ABLE;
        MUSIC_ACTIVE = MUSIC_ABLE;
    }
    1
}

pub fn FreeAudio() {
    unsafe {
        SOUND_ACTIVE = false;
        MUSIC_ACTIVE = false;
    }
}

pub fn GetSoundAble()   -> bool { unsafe { SOUND_ABLE } }
pub fn GetMusicAble()   -> bool { unsafe { MUSIC_ABLE } }
pub fn GetSoundActive() -> bool { unsafe { SOUND_ACTIVE } }
pub fn GetMusicActive() -> bool { unsafe { MUSIC_ACTIVE } }

// Re-export the audio able flags so callers don't need the full path
pub const AUDIO_ABLE_MUSIC_PUB: u16 = AUDIO_ABLE_MUSIC;
pub const AUDIO_ABLE_SOUND_PUB: u16 = AUDIO_ABLE_SOUND;
