use indexmap::IndexMap;
use std::{process::exit, thread::sleep, time::Duration};
use waver_core::{
    Config, ConfigProperty, ConfigPropertyValue, WaveError, WaveXLRDevice, read_config_property,
    write_config_property,
};

mod config;
mod env;

fn main() {
    env_logger::init();

    let mut device = acquire_device();

    log::debug!("loading initial config properties");
    let mut current_properties = match config::read_properties() {
        Ok(value) => value,
        Err(error) => {
            log::error!("failed to read initial config properties: {error}");
            exit(1)
        }
    };

    log::debug!("applying initial device properties");
    if let Err(error) = apply_properties(&mut device, &current_properties) {
        log::error!("failed to apply initial device properties: {error}");
        exit(1)
    };

    let tick_delay = env::tick_delay();
    let tick_error_delay = env::tick_error_delay();

    loop {
        if let Err(error) = tick(&mut device, &mut current_properties) {
            if error.is_disconnected() {
                log::error!("device connection lost: {error}");
                device = acquire_device();
                continue;
            } else {
                log::error!(
                    "error encountered when ticking, resuming tick in {}: {}",
                    tick_error_delay.as_millis(),
                    error
                );
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
                log::warn!(
                    "failed to connect to device, retrying in {}ms",
                    delay.as_millis()
                );
                sleep(delay);
                delay = (delay * 2).min(max_delay);
                continue;
            }
        };

        log::debug!("device connection established");
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
    let current_properties: IndexMap<String, String> = ConfigProperty::ALL
        .iter()
        .map(|property| {
            let value = read_config_property(&config, *property);
            (property.name().to_string(), value.to_string())
        })
        .collect();

    if properties_changed(previous_properties, &current_properties) {
        log::debug!("detected properties changed, writing new properties file");

        if let Err(error) = config::write_properties(&current_properties) {
            log::error!("failed to write properties: {error}");
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
    log::debug!("reading device config to apply update");
    let mut config = device.read::<Config>()?;

    properties
        .iter()
        .filter_map(|(property, value)| {
            let property: ConfigProperty = property.parse().ok()?;
            let value = ConfigPropertyValue::parse_for_property(property, value.to_string())
                .inspect_err(|error|  log::error!(
                    "value \"{value}\" of property \"{property}\" was invalid for the property type: {error}"
                ))
                .ok()?;
            Some((property, value))
        })
        .for_each(|(property, value)| {
            write_config_property(&mut config, property, value)
                // Should never occur provided that ConfigPropertyValue::parse_for_property is
                // implemented correctly
                .expect("invalid value property")
        });

    device.write(&config)?;
    log::debug!("applied updated config to device");

    Ok(())
}
