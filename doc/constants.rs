// BOOLEAN PARAMS

pub const MAIN_MUTE = "main/ch1/mute";
pub const MAIN_MONO = "main/ch1/mono";

pub const CH1_MUTE = "line/ch1/mute";
pub const CH1_SOLO = "line/ch1/solo";

pub const CH2_MUTE = "line/ch2/mute";
pub const CH2_SOLO = "line/ch2/solo";

pub const RETURN1_MUTE = "return/ch1/mute";
pub const RETURN1_SOLO = "return/ch1/solo";
// also return ch2, ch3

pub const FX_RETURN1_MUTE = "fxreturn/ch1/mute";
pub const FX_RETURN1_SOLO = "fxreturn/ch1/solo";


// for enabling effects
pub const CH1_LIMITER_ON = "line/ch1/limit/limiteron"

// FAT CHANNEL

// This is for the fat channel presets that are selectable
// with the wheel
pub const PRESET_SLOT = "line/ch1/activePresetSlotIndex"

pub const CH1_FATCHANNEL_FILTER_HPF = "line/ch1/filter/hpf"; //0.0 is disable
// other stuff is like line/ch1/gate/threshold

// CONTINUOUSLY CHANGED PARAMS (FADERS)

// also for ch2
"line/ch1/preampgain"
"line/ch1/FXA", // fx send

// volumes ch1/ch2/ch3
// 0.735 is unity gain
pub const CH1_VOLUME = "line/ch1/volume";
pub const CH1_PAN = "line/ch1/pan"; // 0.5 is center
pub const RETURN1_VOLUME = "return/ch1/volume";

// stream mix
pub const CH1_VOLUME_MIX1 = "line/ch1/aux1";
pub const CH1_VOLUME_MIX2 = "line/ch1/aux2";

// those changes when the volume knob is turned
// in 3 different settings
pub const SPEAKER_VOLUME = "global/mainOutVolume";
pub const PHONES_VOLUME = "global/phonesVolume";

// changing monitor blend changes those 3 at the same time
pub const GLOBAL_MONITOR_BLEND = "global/monitorBlend";
pub const CH1_MONITOR_BLEND = "line/ch1/monitorBlend";
pub const CH2_MONITOR_BLEND = "line/ch2/monitorBlend";

// This is enabled by clicking the speaker icon
pub const HARDWARE_MUTE = "main/ch1/hardwareMute";


// FX parameters
pub const REVERB_SIZE = "fx/ch1/reverb/size";
pub const REVERB_HP_FREQ = "fx/ch1/reverb/hp_freq";
pub const REVERB_PREDELAY = "fx/ch1/reverb/predelay";

// WEIRD OTHER PARAMS
"global/phonesSrc" // 0.0 for main mix, 0.5 for Stream Mix A, 1.0 for Stream Mix B