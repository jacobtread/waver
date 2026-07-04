use indexmap::IndexMap;
use std::{thread::sleep, time::Duration};
use waver_core::{
    Config, ConfigProperty, ConfigPropertyValue, WaveError, WaveXLRDevice, read_config_property,
    write_config_property,
};

mod config;
mod env;

fn main() -> anyhow::Result<()> {
    let mut device = acquire_device();
    let mut current_properties = config::read_properties()?;
    apply_properties(&mut device, &current_properties)?;

    let tick_delay = env::tick_delay();
    let tick_error_delay = env::tick_error_delay();

    loop {
        if let Err(error) = tick(&mut device, &mut current_properties) {
            if error.is_disconnected() {
                device = acquire_device();
                continue;
            } else {
                sleep(tick_error_delay);
                continue;
            }
        };

        sleep(tick_delay);
    }
}

/// Acquire a device with exponential backup
fn acquire_device() -> WaveXLRDevice {
    let mut delay = Duration::from_secs(1);
    let max_delay = Duration::from_secs(30); // Don't ever sleep longer than 30s

    loop {
        let device = match WaveXLRDevice::connect() {
            Ok(value) => value,
            Err(_) => {
                sleep(delay);
                delay = (delay * 2).min(max_delay);
                continue;
            }
        };

        return device;
    }
}

/// Background tick, reads the current config from the device and saves it to the
/// config file if it has changed
fn tick(
    device: &mut WaveXLRDevice,
    previous_properties: &mut IndexMap<String, String>,
) -> Result<(), WaveError> {
    let config = device.read::<Config>()?;

    let mut current_properties: IndexMap<String, String> = IndexMap::new();

    for property in ConfigProperty::ALL {
        let value = read_config_property(&config, *property);
        current_properties.insert(property.name().to_string(), value.to_string());
    }

    if properties_changed(previous_properties, &current_properties) {
        println!("detected properties changed, writing new properties file");

        if let Err(error) = config::write_properties(&current_properties) {
            eprintln!("failed to write properties: {error}");
        }
    }

    *previous_properties = current_properties;

    Ok(())
}

/// Checks if any properties have changed
fn properties_changed(
    previous_properties: &IndexMap<String, String>,
    next_properties: &IndexMap<String, String>,
) -> bool {
    if previous_properties.len() != next_properties.len() {
        return true;
    }

    previous_properties
        .iter()
        .any(|(key, value)| next_properties.get(key) != Some(value))
}

/// Apply the provided properties onto the device config
fn apply_properties(
    device: &mut WaveXLRDevice,
    properties: &IndexMap<String, String>,
) -> Result<(), WaveError> {
    let mut config = device.read::<Config>()?;

    for (property, value) in properties.iter() {
        let property: ConfigProperty = match property.parse() {
            Ok(value) => value,
            Err(_) => continue,
        };

        let value = match ConfigPropertyValue::parse_for_property(property, value.to_string()) {
            Ok(value) => value,
            Err(error) => {
                eprintln!(
                    "value \"{value}\" of property {property} was invalid for the property type: {error}"
                );
                continue;
            }
        };

        _ = write_config_property(&mut config, property, value);
    }

    device.write(&config)?;

    Ok(())
}
