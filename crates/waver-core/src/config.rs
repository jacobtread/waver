use std::fmt::Display;

use crate::{
    device::{WValue, WValueReadable, WValueWritable},
    format::{decode_signed_q8_8, decode_unsigned_q8_8, encode_signed_q8_8, encode_unsigned_q8_8},
};

const BUFFER_LENGTH: usize = 34;

/// Configuration buffer for reading the Wave XLR config
/// from the device
struct ConfigBuffer {
    buffer: [u8; BUFFER_LENGTH],
}

impl Default for ConfigBuffer {
    fn default() -> Self {
        Self {
            buffer: [0u8; BUFFER_LENGTH],
        }
    }
}

impl ConfigBuffer {
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn read_u8(&self, offset: usize) -> u8 {
        debug_assert!(
            offset < BUFFER_LENGTH,
            "offset must be within bounds of config buffer"
        );
        self.buffer[offset]
    }

    fn write_u8(&mut self, offset: usize, value: u8) {
        debug_assert!(
            offset < BUFFER_LENGTH,
            "offset must be within bounds of config buffer"
        );
        self.buffer[offset] = value;
    }

    fn write_signed_q8_8(&mut self, offset: usize, value: f32) {
        debug_assert!(
            offset < BUFFER_LENGTH && offset + 1 < BUFFER_LENGTH,
            "offset must be within bounds of config buffer"
        );

        let encoded = encode_signed_q8_8(value);
        self.buffer[offset..=offset + 1].copy_from_slice(&encoded);
    }

    fn write_unsigned_q8_8(&mut self, offset: usize, value: f32) {
        debug_assert!(
            offset < BUFFER_LENGTH && offset + 1 < BUFFER_LENGTH,
            "offset must be within bounds of config buffer"
        );

        let encoded = encode_unsigned_q8_8(value);
        self.buffer[offset..=offset + 1].copy_from_slice(&encoded);
    }

    fn read_signed_q8_8(&self, offset: usize) -> f32 {
        debug_assert!(
            offset < BUFFER_LENGTH && offset + 1 < BUFFER_LENGTH,
            "offset must be within bounds of config buffer"
        );

        let mut bytes = [0u8; 2];
        let bytes_ref = &self.buffer[offset..=offset + 1];
        bytes.copy_from_slice(bytes_ref);
        decode_signed_q8_8(bytes)
    }

    fn read_unsigned_q8_8(&self, offset: usize) -> f32 {
        debug_assert!(
            offset < BUFFER_LENGTH && offset + 1 < BUFFER_LENGTH,
            "offset must be within bounds of config buffer"
        );

        let mut bytes = [0u8; 2];
        let bytes_ref = &self.buffer[offset..=offset + 1];
        bytes.copy_from_slice(bytes_ref);
        decode_unsigned_q8_8(bytes)
    }
}

#[derive(Default)]
pub struct Config {
    buffer: ConfigBuffer,
}

const OFFSET_GAIN: usize = 0;
const OFFSET_MUTE: usize = 4;
const OFFSET_HEADPHONE_VOLUME: usize = 9;
const OFFSET_MIC_MIX: usize = 12;
const OFFSET_VOLUME_SELECT: usize = 14;
const OFFSET_LOW_IMPEDANCE: usize = 33;

#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord, Clone, Copy)]
pub enum VolumeSelectMode {
    #[default]
    Gain,
    Headphones,
    Mix,
    Unknown(u8),
}

impl Display for VolumeSelectMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            VolumeSelectMode::Gain => "Gain",
            VolumeSelectMode::Headphones => "Headphones",
            VolumeSelectMode::Mix => "Mix",
            VolumeSelectMode::Unknown(value) => {
                return write!(f, "Unknown({value})");
            }
        })
    }
}

impl Config {
    fn buffer_mut(&mut self) -> &mut [u8] {
        self.buffer.buffer_mut()
    }

    fn buffer(&self) -> &[u8] {
        self.buffer.buffer()
    }

    pub fn get_mute(&self) -> bool {
        self.buffer.read_u8(OFFSET_MUTE) != 0
    }

    pub fn set_mute(&mut self, value: bool) {
        self.buffer.write_u8(OFFSET_MUTE, value as u8);
    }

    pub fn get_mic_mix(&self) -> f32 {
        self.buffer.read_unsigned_q8_8(OFFSET_MIC_MIX)
    }

    pub fn set_mic_mix(&mut self, value: f32) {
        debug_assert!(
            (0.0..100.0).contains(&value),
            "mic mix value must be within 0.0 and 100.0"
        );
        self.buffer.write_unsigned_q8_8(OFFSET_MIC_MIX, value);
    }

    pub fn get_gain(&self) -> f32 {
        self.buffer.read_signed_q8_8(OFFSET_GAIN)
    }

    pub fn set_gain(&mut self, value: f32) {
        self.buffer.write_signed_q8_8(OFFSET_GAIN, value);
    }

    pub fn get_headphone_volume(&self) -> f32 {
        self.buffer.read_signed_q8_8(OFFSET_HEADPHONE_VOLUME)
    }

    pub fn set_headphone_volume(&mut self, value: f32) {
        self.buffer
            .write_signed_q8_8(OFFSET_HEADPHONE_VOLUME, value);
    }

    pub fn get_low_impedance(&self) -> bool {
        self.buffer.read_u8(OFFSET_LOW_IMPEDANCE) != 0
    }

    pub fn set_low_impedance(&mut self, value: bool) {
        self.buffer.write_u8(OFFSET_LOW_IMPEDANCE, value as u8);
    }

    pub fn get_volume_select_mode(&self) -> VolumeSelectMode {
        let value = self.buffer.read_u8(OFFSET_VOLUME_SELECT);
        match value {
            1 => VolumeSelectMode::Gain,
            2 => VolumeSelectMode::Headphones,
            3 => VolumeSelectMode::Mix,
            value => VolumeSelectMode::Unknown(value),
        }
    }
}

impl WValue for Config {
    const VALUE: u16 = 0x0000;
}

impl WValueReadable for Config {
    const REQUIRE_EXACT: bool = true;

    fn wave_buffer_mut(&mut self) -> &mut [u8] {
        self.buffer_mut()
    }
}

impl WValueWritable for Config {
    fn wave_buffer(&self) -> &[u8] {
        self.buffer()
    }
}
