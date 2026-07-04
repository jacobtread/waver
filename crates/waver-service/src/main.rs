use std::{
    fs::{File, exists},
    io::{BufRead, BufReader},
    thread::sleep,
    time::Duration,
};

use indexmap::IndexMap;
use waver_core::{
    Config, ConfigProperty, ConfigPropertyValue, WaveError, WaveXLRDevice, read_config_property,
    write_config_property,
};

static CONFIG_FILE: &str = "/etc/waver.conf";

const ENV_WAVER_TICK_DELAY_MS: &str = "WAVER_TICK_DELAY_MS";
const ENV_WAVER_TICK_ERROR_DELAY_MS: &str = "WAVER_TICK_ERROR_DELAY_MS";

fn tick_delay() -> Option<Duration> {
    std::env::var(ENV_WAVER_TICK_DELAY_MS)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
}

fn tick_error_delay() -> Option<Duration> {
    std::env::var(ENV_WAVER_TICK_ERROR_DELAY_MS)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_millis)
}

fn main() -> anyhow::Result<()> {
    let mut device = acquire_device();
    let mut current_properties = read_properties()?;
    apply_properties(&mut device, &current_properties)?;

    let tick_delay = tick_delay().unwrap_or(Duration::from_millis(100));
    let tick_error_delay = tick_error_delay().unwrap_or(Duration::from_millis(500));

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

        if let Err(error) = write_properties(&current_properties) {
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
    let keys = previous_properties.keys().chain(next_properties.keys());

    for key in keys {
        let left = previous_properties.get(key);
        let right = next_properties.get(key);

        if left != right {
            return true;
        }
    }

    false
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

/// Read the properties from the config file
fn read_properties() -> std::io::Result<IndexMap<String, String>> {
    if !exists(CONFIG_FILE)? {
        return Ok(IndexMap::new());
    }

    let file = File::open(CONFIG_FILE)?;
    let reader = BufReader::new(file);
    let lines = reader.lines();

    let mut properties = IndexMap::new();

    for line in lines {
        let line = line?;
        if line.is_empty() {
            continue;
        }

        let (property, value) = match line.split_once('=') {
            Some(value) => value,
            None => continue,
        };

        properties.insert(property.to_string(), value.to_string());
    }

    Ok(properties)
}

/// Write the properties to the config file
fn write_properties(properties: &IndexMap<String, String>) -> std::io::Result<()> {
    use std::fmt::Write;

    let mut output = String::new();
    for (key, value) in properties.iter() {
        writeln!(&mut output, "{key}={value}").expect("failed to write string");
    }

    std::fs::write(CONFIG_FILE, output)?;
    Ok(())
}
