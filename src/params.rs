//! Parameter paths observed on a Revelator io24 (see `doc/constants.rs` for
//! the raw notes). Any of these can be sent in a PV packet; booleans are
//! `1.0`/`0.0`. Run `app dump-state` to list everything your device exposes.

// Main bus. `MAIN_MUTE` is the software mute that Universal Control never
// shows for the main bus; `HARDWARE_MUTE` is what the physical mute button
// (and the speaker icon in UC) toggles.
pub const MAIN_MUTE: &str = "main/ch1/mute";
pub const HARDWARE_MUTE: &str = "main/ch1/hardwareMute";
pub const MAIN_MONO: &str = "main/ch1/mono";
pub const MAIN_VOLUME: &str = "main/ch1/volume";

pub const CH1_MUTE: &str = "line/ch1/mute";
pub const CH1_SOLO: &str = "line/ch1/solo";
pub const CH2_MUTE: &str = "line/ch2/mute";
pub const CH2_SOLO: &str = "line/ch2/solo";

/// 0.735 is unity gain.
pub const CH1_VOLUME: &str = "line/ch1/volume";
/// 0.5 is centre.
pub const CH1_PAN: &str = "line/ch1/pan";

pub const SPEAKER_VOLUME: &str = "global/mainOutVolume";
pub const PHONES_VOLUME: &str = "global/phonesVolume";
pub const PHONES_MUTE: &str = "global/phonesMute";
/// 0.0 main mix, 0.5 stream mix A, 1.0 stream mix B.
pub const PHONES_SRC: &str = "global/phonesSrc";
pub const MONITOR_BLEND: &str = "global/monitorBlend";

/// 0.735 on a volume parameter is unity gain.
pub const UNITY_GAIN: f32 = 0.735;
