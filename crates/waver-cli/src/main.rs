use waver_core::{Config, WaveXLRDevice};

fn main() -> anyhow::Result<()> {
    let mut device = WaveXLRDevice::connect()?;
    let config = device.read::<Config>()?;

    let mute = config.get_mute();
    let gain = config.get_gain();
    let headphone = config.get_headphone_volume();
    let low_impedance = config.get_low_impedance();
    let volume_select_mode = config.get_volume_select_mode();
    let mic_mix = config.get_mic_mix();

    println!("Muted = {mute}");
    println!("Gain = {gain}");
    println!("Headphone = {headphone}");
    println!("Low Impedance = {low_impedance}");
    println!("Volume Select mode = {volume_select_mode}");
    println!("Current mic mix = {mic_mix}");

    Ok(())
}
