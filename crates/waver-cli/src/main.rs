use clap::{Parser, Subcommand, ValueEnum};
use indexmap::IndexMap;
use serde::Serialize;
use waver_core::{
    Config, ConfigProperty, ConfigPropertyValue, DeviceInfo, Meters, WaveXLRDevice,
    read_config_property, write_config_property,
};

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
        property: CliConfigProperty,
    },
    /// Read a config value from the device
    WriteConfig {
        /// Property to write to
        #[arg(short, long, value_enum)]
        property: CliConfigProperty,
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
    value: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum CliConfigProperty {
    Mute,
    Gain,
    HeadphoneVolume,
    LowImpedance,
    VolumeSelectMode,
    MicMix,
}

impl From<CliConfigProperty> for ConfigProperty {
    fn from(value: CliConfigProperty) -> Self {
        match value {
            CliConfigProperty::Mute => ConfigProperty::Mute,
            CliConfigProperty::Gain => ConfigProperty::Gain,
            CliConfigProperty::HeadphoneVolume => ConfigProperty::HeadphoneVolume,
            CliConfigProperty::LowImpedance => ConfigProperty::LowImpedance,
            CliConfigProperty::VolumeSelectMode => ConfigProperty::VolumeSelectMode,
            CliConfigProperty::MicMix => ConfigProperty::MicMix,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Args::parse();

    match cli.command {
        Commands::ReadConfig { property } => {
            let property = ConfigProperty::from(property);
            let mut device = WaveXLRDevice::connect()?;
            let config = device.read::<Config>()?;
            let value = read_config_property(&config, property);
            match cli.format {
                OutputFormat::Plain => {
                    println!("{value}")
                }
                OutputFormat::Json => {
                    let value = ConfigValueJson {
                        value: value.to_string(),
                    };
                    let value = serde_json::to_string(&value)?;
                    println!("{value}")
                }
            }
        }
        Commands::WriteConfig { property, value } => {
            let property = ConfigProperty::from(property);
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
                    let value = ConfigValueJson {
                        value: value.to_string(),
                    };
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
                let name = property.name();
                let value = read_config_property(&config, *property);
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
