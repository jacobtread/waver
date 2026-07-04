use std::{
    fmt::Display,
    num::ParseFloatError,
    str::{FromStr, ParseBoolError},
};

use serde::Serialize;
use thiserror::Error;

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

#[derive(Debug, PartialEq, Eq, Default, PartialOrd, Ord, Clone, Copy, Serialize)]
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

#[derive(Debug)]
pub struct InvalidVolumeSelectMode;

impl Display for InvalidVolumeSelectMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid volume select mode")
    }
}

impl std::error::Error for InvalidVolumeSelectMode {}

impl FromStr for VolumeSelectMode {
    type Err = InvalidVolumeSelectMode;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Gain" => VolumeSelectMode::Gain,
            "Headphones" => VolumeSelectMode::Headphones,
            "Mix" => VolumeSelectMode::Mix,
            value => {
                let value: u8 = value.parse().map_err(|_| InvalidVolumeSelectMode)?;
                VolumeSelectMode::Unknown(value)
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

    pub fn set_volume_select_mode(&mut self, value: VolumeSelectMode) {
        let value = match value {
            VolumeSelectMode::Gain => 1,
            VolumeSelectMode::Headphones => 2,
            VolumeSelectMode::Mix => 3,
            VolumeSelectMode::Unknown(value) => value,
        };

        self.buffer.write_u8(OFFSET_VOLUME_SELECT, value);
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigProperty {
    Mute,
    Gain,
    HeadphoneVolume,
    LowImpedance,
    VolumeSelectMode,
    MicMix,
}

impl Display for ConfigProperty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl ConfigProperty {
    pub fn name(&self) -> &str {
        match self {
            ConfigProperty::Mute => "mute",
            ConfigProperty::Gain => "gain",
            ConfigProperty::HeadphoneVolume => "headphone-volume",
            ConfigProperty::LowImpedance => "low-impedance",
            ConfigProperty::VolumeSelectMode => "volume-select-mode",
            ConfigProperty::MicMix => "mic-mix",
        }
    }
}

#[derive(Debug, Error)]
#[error("unknown config property field")]
pub struct UnknownConfigPropertyError;

impl FromStr for ConfigProperty {
    type Err = UnknownConfigPropertyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "mute" => ConfigProperty::Mute,
            "gain" => ConfigProperty::Gain,
            "headphone-volume" => ConfigProperty::HeadphoneVolume,
            "low-impedance" => ConfigProperty::LowImpedance,
            "volume-select-mode" => ConfigProperty::VolumeSelectMode,
            "mic-mix" => ConfigProperty::MicMix,
            _ => return Err(UnknownConfigPropertyError),
        })
    }
}

impl ConfigProperty {
    pub const ALL: &[ConfigProperty] = &[
        ConfigProperty::Mute,
        ConfigProperty::Gain,
        ConfigProperty::HeadphoneVolume,
        ConfigProperty::LowImpedance,
        ConfigProperty::VolumeSelectMode,
        ConfigProperty::MicMix,
    ];
}

#[derive(Debug, Clone, Serialize)]
pub enum ConfigPropertyValue {
    Float(f32),
    Boolean(bool),
    VolumeSelectMode(VolumeSelectMode),
}

#[derive(Debug, Error)]
pub enum ConfigPropertyError {
    #[error(transparent)]
    ParseBoolean(#[from] ParseBoolError),

    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),

    #[error(transparent)]
    VolumeSelectMode(#[from] InvalidVolumeSelectMode),
}

impl ConfigPropertyValue {
    pub fn parse_for_property(
        property: ConfigProperty,
        value: String,
    ) -> Result<ConfigPropertyValue, ConfigPropertyError> {
        Ok(match property {
            ConfigProperty::Mute | ConfigProperty::LowImpedance => {
                let value: bool = value.parse()?;
                ConfigPropertyValue::Boolean(value)
            }
            ConfigProperty::Gain | ConfigProperty::HeadphoneVolume | ConfigProperty::MicMix => {
                let value: f32 = value.parse()?;
                ConfigPropertyValue::Float(value)
            }
            ConfigProperty::VolumeSelectMode => {
                let value: VolumeSelectMode = value.parse()?;
                ConfigPropertyValue::VolumeSelectMode(value)
            }
        })
    }
}

impl Display for ConfigPropertyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigPropertyValue::Float(value) => value.fmt(f),
            ConfigPropertyValue::Boolean(value) => value.fmt(f),
            ConfigPropertyValue::VolumeSelectMode(value) => value.fmt(f),
        }
    }
}

impl From<f32> for ConfigPropertyValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
    }
}

impl From<bool> for ConfigPropertyValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<VolumeSelectMode> for ConfigPropertyValue {
    fn from(value: VolumeSelectMode) -> Self {
        Self::VolumeSelectMode(value)
    }
}

pub fn read_config_property(config: &Config, property: ConfigProperty) -> ConfigPropertyValue {
    match property {
        ConfigProperty::Mute => config.get_mute().into(),
        ConfigProperty::Gain => config.get_gain().into(),
        ConfigProperty::HeadphoneVolume => config.get_headphone_volume().into(),
        ConfigProperty::LowImpedance => config.get_low_impedance().into(),
        ConfigProperty::VolumeSelectMode => config.get_volume_select_mode().into(),
        ConfigProperty::MicMix => config.get_mic_mix().into(),
    }
}

#[derive(Debug, Error)]
#[error("unsupported property value combination")]
pub struct UnsupportedValueCombination;

pub fn write_config_property(
    config: &mut Config,
    property: ConfigProperty,
    value: ConfigPropertyValue,
) -> Result<(), UnsupportedValueCombination> {
    match (property, value) {
        (ConfigProperty::Mute, ConfigPropertyValue::Boolean(value)) => config.set_mute(value),
        (ConfigProperty::Gain, ConfigPropertyValue::Float(value)) => config.set_gain(value),
        (ConfigProperty::HeadphoneVolume, ConfigPropertyValue::Float(value)) => {
            config.set_headphone_volume(value);
        }
        (ConfigProperty::LowImpedance, ConfigPropertyValue::Boolean(value)) => {
            config.set_low_impedance(value);
        }
        (ConfigProperty::VolumeSelectMode, ConfigPropertyValue::VolumeSelectMode(value)) => {
            config.set_volume_select_mode(value);
        }
        (ConfigProperty::MicMix, ConfigPropertyValue::Float(value)) => config.set_mic_mix(value),

        _ => return Err(UnsupportedValueCombination),
    }

    Ok(())
}
