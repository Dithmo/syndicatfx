// bfsoundlib — Bullfrog sound library, Rust port.
// Audio backend: OpenAL (via openal-sys) + OGG Vorbis (lewton).
// For now this is a functional stub; AIL/AIL2OAL integration follows.

pub mod audio;
pub mod music;
pub mod samples;

pub use audio::*;
pub use music::*;
pub use samples::*;
