pub fn encode_signed_q8_8(value: f32) -> [u8; 2] {
    let scaled = (value * 256.0).round();
    let value = scaled.clamp(-32768.0, 32767.0) as i16;
    value.to_le_bytes()
}

pub fn decode_signed_q8_8(value: [u8; 2]) -> f32 {
    let value = i16::from_le_bytes(value);
    value as f32 / 256.0
}

pub fn encode_unsigned_q8_8(value: f32) -> [u8; 2] {
    let scaled = (value * 256.0).round();
    let value = scaled.clamp(-32768.0, 32767.0) as u16;
    value.to_le_bytes()
}

pub fn decode_unsigned_q8_8(value: [u8; 2]) -> f32 {
    let value = u16::from_le_bytes(value);
    value as f32 / 256.0
}
