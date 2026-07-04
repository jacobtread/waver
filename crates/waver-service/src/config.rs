use indexmap::IndexMap;
use std::fs::exists;

static CONFIG_FILE: &str = "/etc/waver.conf";

/// Read the properties from the config file
pub fn read_properties() -> std::io::Result<IndexMap<String, String>> {
    if !exists(CONFIG_FILE)? {
        return Ok(IndexMap::new());
    }

    let text = std::fs::read_to_string(CONFIG_FILE)?;
    let properties: IndexMap<String, String> = text
        .lines()
        .filter(|line| !line.is_empty())
        .filter_map(|line| line.split_once('='))
        .map(|(property, value)| (property.to_string(), value.to_string()))
        .collect();

    Ok(properties)
}

/// Write the properties to the config file
pub fn write_properties(properties: &IndexMap<String, String>) -> std::io::Result<()> {
    let output = properties
        .iter()
        .fold(String::new(), |mut output, (key, value)| {
            output.push_str(key);
            output.push('=');
            output.push_str(value);
            output.push('\n');
            output
        });

    std::fs::write(CONFIG_FILE, output)?;
    Ok(())
}
