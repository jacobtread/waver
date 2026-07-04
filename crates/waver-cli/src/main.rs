use std::{fmt::Display, str::FromStr};

use anyhow::Context;
use clap::{Parser, Subcommand, ValueEnum};
use indexmap::IndexMap;
use serde::Serialize;
use waver_core::{Config, DeviceInfo, Meters, VolumeSelectMode, WaveXLRDevice};

/// Simple tool for working with the Wave XLR over USB
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Output format to use when providing values back
    #[arg(short, long, value_enum, default_value_t = OutputFormat::Plain)]
    format: OutputFormat,

    /// Command to run
    #[command(subcommand)]
    command: Commands,
}

#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    /// Plain text output format
    #[default]
    Plain,
    /// JSON output format
    Json,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Read a config value from the device
    ReadConfig {
        /// Specific property to get the value of
        #[arg(short, long, value_enum)]
        property: ConfigProperty,
    },
    /// Read a config value from the device
    WriteConfig {
        /// Property to write to
        #[arg(short, long, value_enum)]
        property: ConfigProperty,
        /// Value to write to the property
        #[arg(short, long)]
        value: String,
    },
    /// Dump out all of the config properties
    DumpConfig,
    /// Dump out the device info
    DeviceInfo,
    /// Read the L&R meters value
    Meters,
}

#[derive(Serialize)]
pub struct ConfigValueJson {
    value: ConfigPropertyValue,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ConfigProperty {
    Mute,
    Gain,
    HeadphoneVolume,
    LowImpedance,
    VolumeSelectMode,
    MicMix,
}

impl Serialize for ConfigProperty {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let possible_value = self
            .to_possible_value()
            .expect("should always have a possible value");
        let name = possible_value.get_name();
        name.serialize(serializer)
    }
}

impl ConfigProperty {
    const ALL: &[ConfigProperty] = &[
        ConfigProperty::Mute,
        ConfigProperty::Gain,
        ConfigProperty::HeadphoneVolume,
        ConfigProperty::LowImpedance,
        ConfigProperty::VolumeSelectMode,
        ConfigProperty::MicMix,
    ];
}

#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
enum ConfigPropertyValue {
    Float(f32),
    Boolean(bool),
    String(String),
}

impl ConfigPropertyValue {
    pub fn parse_for_property(
        property: ConfigProperty,
        value: String,
    ) -> anyhow::Result<ConfigPropertyValue> {
        Ok(match property {
            ConfigProperty::Mute | ConfigProperty::LowImpedance => {
                let value: bool = value.parse()?;
                ConfigPropertyValue::Boolean(value)
            }
            ConfigProperty::Gain | ConfigProperty::HeadphoneVolume | ConfigProperty::MicMix => {
                let value: f32 = value.parse()?;
                ConfigPropertyValue::Float(value)
            }
            ConfigProperty::VolumeSelectMode => ConfigPropertyValue::String(value),
        })
    }
}

impl Display for ConfigPropertyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigPropertyValue::Float(value) => value.fmt(f),
            ConfigPropertyValue::Boolean(value) => value.fmt(f),
            ConfigPropertyValue::String(value) => value.fmt(f),
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

impl From<String> for ConfigPropertyValue {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

fn read_config_property(
    config: &Config,
    property: ConfigProperty,
) -> anyhow::Result<ConfigPropertyValue> {
    Ok(match property {
        ConfigProperty::Mute => config.get_mute().into(),
        ConfigProperty::Gain => config.get_gain().into(),
        ConfigProperty::HeadphoneVolume => config.get_headphone_volume().into(),
        ConfigProperty::LowImpedance => config.get_low_impedance().into(),
        ConfigProperty::VolumeSelectMode => config.get_volume_select_mode().to_string().into(),
        ConfigProperty::MicMix => config.get_mic_mix().into(),
    })
}

fn write_config_property(
    config: &mut Config,
    property: ConfigProperty,
    value: ConfigPropertyValue,
) -> anyhow::Result<()> {
    match (property, value) {
        (ConfigProperty::Mute, ConfigPropertyValue::Boolean(value)) => config.set_mute(value),
        (ConfigProperty::Gain, ConfigPropertyValue::Float(value)) => config.set_gain(value),
        (ConfigProperty::HeadphoneVolume, ConfigPropertyValue::Float(value)) => {
            config.set_headphone_volume(value);
        }
        (ConfigProperty::LowImpedance, ConfigPropertyValue::Boolean(value)) => {
            config.set_low_impedance(value);
        }
        (ConfigProperty::VolumeSelectMode, ConfigPropertyValue::String(value)) => {
            let value = VolumeSelectMode::from_str(&value).context("invalid volume select mode")?;
            config.set_volume_select_mode(value);
        }
        (ConfigProperty::MicMix, ConfigPropertyValue::Float(value)) => config.set_mic_mix(value),

        _ => anyhow::bail!("unsupported property value combination"),
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let cli = Args::parse();

    match cli.command {
        Commands::ReadConfig { property } => {
            let mut device = WaveXLRDevice::connect()?;
            let config = device.read::<Config>()?;
            let value = read_config_property(&config, property)?;
            match cli.format {
                OutputFormat::Plain => {
                    println!("{value}")
                }
                OutputFormat::Json => {
                    let value = ConfigValueJson { value };
                    let value = serde_json::to_string(&value)?;
                    println!("{value}")
                }
            }
        }
        Commands::WriteConfig { property, value } => {
            let value = ConfigPropertyValue::parse_for_property(property, value)?;
            let mut device = WaveXLRDevice::connect()?;
            let mut config = device.read::<Config>()?;
            write_config_property(&mut config, property, value.clone())?;
            device.write(&config)?;

            match cli.format {
                OutputFormat::Plain => {
                    println!("{value}")
                }
                OutputFormat::Json => {
                    let value = ConfigValueJson { value };
                    let value = serde_json::to_string(&value)?;
                    println!("{value}")
                }
            }
        }
        Commands::DumpConfig => {
            let mut device = WaveXLRDevice::connect()?;
            let config = device.read::<Config>()?;
            let mut properties: IndexMap<String, String> = IndexMap::new();

            for property in ConfigProperty::ALL {
                let possible_value = property
                    .to_possible_value()
                    .expect("should always have a possible value");
                let name = possible_value.get_name();
                let value = read_config_property(&config, *property)?;
                properties.insert(name.to_string(), value.to_string());
            }

            match cli.format {
                OutputFormat::Plain => {
                    for (key, value) in properties.iter() {
                        println!("{key}={value}");
                    }
                }
                OutputFormat::Json => {
                    let value = serde_json::to_string(&properties)?;
                    println!("{value}")
                }
            }
        }
        Commands::DeviceInfo => {
            let mut device = WaveXLRDevice::connect()?;
            let device_info = device.read::<DeviceInfo>()?;
            let mut properties: IndexMap<String, String> = IndexMap::new();

            let api_version = device_info.api_version();
            let firmware_version = device_info.firmware_version();
            let serial_number = device_info
                .serial_number()
                .unwrap_or_else(|_| "failed to read serial number".to_string());

            properties.insert(
                "api-version".to_string(),
                format!("{}.{}", api_version.0, api_version.1),
            );
            properties.insert(
                "firmware-version".to_string(),
                format!(
                    "{}.{}.{}",
                    firmware_version.0, firmware_version.1, firmware_version.2
                ),
            );
            properties.insert("serial-number".to_string(), serial_number);

            match cli.format {
                OutputFormat::Plain => {
                    for (key, value) in properties.iter() {
                        println!("{key}={value}");
                    }
                }
                OutputFormat::Json => {
                    let value = serde_json::to_string(&properties)?;
                    println!("{value}")
                }
            }
        }
        Commands::Meters => {
            let mut device = WaveXLRDevice::connect()?;
            let meters = device.read::<Meters>()?;
            let mut properties: IndexMap<String, String> = IndexMap::new();

            properties.insert("left".to_string(), meters.read_left().to_string());
            properties.insert("right".to_string(), meters.read_right().to_string());

            match cli.format {
                OutputFormat::Plain => {
                    for (key, value) in properties.iter() {
                        println!("{key}={value}");
                    }
                }
                OutputFormat::Json => {
                    let value = serde_json::to_string(&properties)?;
                    println!("{value}")
                }
            }
        }
    }

    Ok(())
}
