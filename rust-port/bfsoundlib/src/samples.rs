// Sample playback — mirrors ssampply.h / PlaySampleFromAddress etc.

use crate::audio::{GetSoundAble, GetSoundActive};

pub const FULL_VOL: i32  = 127;
pub const EQUL_PAN: i32  = 64;
pub const NORM_PTCH: i32 = 100;

pub fn PlaySampleFromAddress(_channel: i32, _smp_id: u8, _vol: i32, _pan: i32,
                              _pitch: i32, _flags: u32, _priority: u32,
                              _data: *const u8) {
    if !GetSoundAble() || !GetSoundActive() { return; }
    // OpenAL dispatch goes here.
}

pub fn PauseAllSamples() {}
pub fn ResumeAllSamples() {}

pub fn FreeSound() {}
