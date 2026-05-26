// sound.rs — mirrors sound.c: sample/music management wrappers.

use bfsoundlib::audio::{AudioInitOptions, InitAudio, FreeAudio,
                         GetSoundAble, GetSoundActive,
                         AUDIO_ABLE_MUSIC, AUDIO_ABLE_SOUND};
use bfsoundlib::music::{InitMIDI, ShutdownMIDI,
                         BFMidiPauseSong, BFMidiResumeSong};
use bfsoundlib::samples::{PlaySampleFromAddress, PauseAllSamples, ResumeAllSamples,
                            FULL_VOL, EQUL_PAN, NORM_PTCH};
use crate::globals::*;

// Re-export music functions so game.rs can call them by name
pub use bfsoundlib::music::{BFMidiStartMusic, BFMidiStopMusic, BFMidiIsMusicPlaying};
pub use bfsoundlib::audio::GetMusicAble;

// ---- BFSample / BFSampleStatus (mirroring sound.h structs) ----------------

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct BFSample {
    pub field_0:      i32,
    pub field_4:      i32,
    pub field_8:      i32,
    pub field_c:      i16,
    pub data_shifted: i32,
    pub data_start:   *mut u8,
    pub field_16:     i16,
    pub field_18:     i32,
    pub field_1c:     i16,
    pub field_1e:     u8,   // priority
    pub field_1f:     u8,   // allow-finish flag
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct BFSampleStatus {
    pub field_0: u8,
    pub field_1: u8,
}

// ---- Audio options initialisation -----------------------------------------

pub static mut AUD_OPTS: AudioInitOptions = AudioInitOptions {
    sound_data_path:             b"sound\0".as_ptr() as *const _,
    sound_driver_path:           b"data\0".as_ptr()  as *const _,
    ini_path:                    b".\0".as_ptr()      as *const _,
    auto_scan:                   1,
    stereo_option:               1,
    disable_load_sounds:         1,
    disable_load_music:          1,
    init_redbook_audio:          0,
    use_current_awe32_soundfont: 1,
    able_flags: AUDIO_ABLE_MUSIC | AUDIO_ABLE_SOUND,
    sound_type: 1622,
    max_samples: 10,
};

pub fn audio_options_set_default() {
    unsafe {
        AUD_OPTS.sound_data_path = b"sound\0".as_ptr() as *const _;
        AUD_OPTS.sound_driver_path = b"data\0".as_ptr() as *const _;
        AUD_OPTS.ini_path = b".\0".as_ptr() as *const _;
        AUD_OPTS.auto_scan = 1;
        AUD_OPTS.stereo_option = 1;
        AUD_OPTS.disable_load_sounds = 1;
        AUD_OPTS.disable_load_music  = 1;
        AUD_OPTS.init_redbook_audio  = 0;
        AUD_OPTS.use_current_awe32_soundfont = 1;
        AUD_OPTS.able_flags = AUDIO_ABLE_MUSIC | AUDIO_ABLE_SOUND;
        AUD_OPTS.sound_type  = 1622;
        AUD_OPTS.max_samples = 10;
    }
}

pub fn init_audio() {
    unsafe {
        InitAudio(&AUD_OPTS as *const _);
        if GetMusicAble() {
            let bank = b"sound/syngame.xmi\0".as_ptr() as *const _;
            let drv  = b"data/gamefm.dll\0".as_ptr()   as *const _;
            InitMIDI(bank, drv as *mut _, SNDCARD_IRQ, SNDCARD_DMA, SNDCARD_IOADDR);
        }
    }
}

// ---- Sample bank setup -----------------------------------------------------

pub fn sound_bank_setup() {
    unsafe {
        if SMPTABLE.is_null() || !GetSoundAble() { return; }
        let mut p = SMPTABLE.add(1);
        while p < SMPTABLE_END {
            let voc_data = SMPDATA.add((*p).data_start as usize);
            (*p).data_shifted = (voc_data as u32).wrapping_shl(12) as i32;
            (*p).data_start = voc_data;
            p = p.add(1);
        }
    }
}

// ---- Sample status bookkeeping ---------------------------------------------

pub fn clear_bf_sample_status() {
    unsafe {
        for s in SAMPLE_STATUS.iter_mut() {
            s.field_0 = 0;
            s.field_1 = 0;
        }
    }
}

pub fn set_bf_sample_status(smp_id: u8, stat: u8) {
    unsafe {
        let p_smp    = &*SMPTABLE.add(smp_id as usize);
        let p_status = &mut SAMPLE_STATUS[smp_id as usize];
        if p_smp.field_1e == 0 { return; }
        if p_status.field_0 == 0 || stat > p_status.field_1 {
            p_status.field_1 = stat;
            p_status.field_0 = 1;
        }
    }
}

// ---- Sample playback -------------------------------------------------------

pub fn SetBFSampleStatus(smp_id: u8, stat: u8) {
    set_bf_sample_status(smp_id, stat);
}

pub fn BFPlaySample(smp_id: u8) {
    if !GetSoundAble() || !GetSoundActive() { return; }
    unsafe {
        let p_smp = &*SMPTABLE.add(smp_id as usize);
        PlaySampleFromAddress(0, smp_id, FULL_VOL, EQUL_PAN, NORM_PTCH,
                              0, 1, p_smp.data_start as *const u8);
    }
}

pub fn BFSonundUnkn1() {
    use bfsoundlib::audio::GetSoundAble as _GSA;
    use bfsoundlib::audio::GetSoundActive as _GSAC;
    if !_GSA() || !_GSAC() { return; }
    unsafe {
        BYTE_5BBE8 = 0;
        let mut sel_smp_id: i16 = 0;

        for smp_id in 1u8..=255 {
            let p_smp    = &*SMPTABLE.add(smp_id as usize);
            let p_status = &mut SAMPLE_STATUS[smp_id as usize];
            if p_status.field_0 == 0 { continue; }
            p_status.field_0 -= 1;
            let v6 = p_smp.field_1e;
            if v6 > BYTE_5BBE8
                || (v6 == BYTE_5BBE8 && p_smp.field_1f == 0)
                || (p_smp.field_1e == BYTE_5BBE8 && p_status.field_1 > BYTE_5BBE9)
            {
                sel_smp_id = smp_id as i16;
                BYTE_5BBE8 = p_smp.field_1e;
                BYTE_5BBE9 = p_status.field_1;
            }
        }

        if sel_smp_id != 0 {
            let id = sel_smp_id as usize;
            let p_smp = &*SMPTABLE.add(id);
            PlaySampleFromAddress(0, id as u8, 127, 64, 100,
                                  0, 1, p_smp.data_start as *const u8);
            SAMPLE_STATUS[id].field_0 = 0;
        }
    }
}

pub fn BFSoundPause() {
    if !GetSoundAble() || !GetSoundActive() { return; }
    PauseAllSamples();
}

pub fn BFSoundResume() {
    if !GetSoundAble() || !GetSoundActive() { return; }
    ResumeAllSamples();
}
