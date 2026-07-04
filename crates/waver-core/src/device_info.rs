use crate::device::{WValue, WValueReadable, WValueWritable};

const BUFFER_LENGTH: usize = 51;

struct DeviceInfoBuffer {
    buffer: [u8; BUFFER_LENGTH],
}

impl Default for DeviceInfoBuffer {
    fn default() -> Self {
        Self {
            buffer: [0u8; BUFFER_LENGTH],
        }
    }
}

impl DeviceInfoBuffer {
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

    fn read_slice(&self, offset: usize, length: usize) -> &[u8] {
        debug_assert!(
            offset < BUFFER_LENGTH && offset + length < BUFFER_LENGTH,
            "offset must be within bounds of config buffer"
        );
        &self.buffer[offset..offset + length]
    }
}

#[derive(Default)]
pub struct DeviceInfo {
    buffer: DeviceInfoBuffer,
}

impl DeviceInfo {
    fn buffer_mut(&mut self) -> &mut [u8] {
        self.buffer.buffer_mut()
    }

    fn buffer(&self) -> &[u8] {
        self.buffer.buffer()
    }

    pub fn api_version(&self) -> (u8, u8) {
        let major = self.buffer.read_u8(0);
        let minor = self.buffer.read_u8(1);
        (major, minor)
    }

    pub fn firmware_version(&self) -> (u8, u8, u8) {
        let major = self.buffer.read_u8(6);
        let minor = self.buffer.read_u8(7);
        let patch = self.buffer.read_u8(8);
        (major, minor, patch)
    }

    pub fn serial_number(&self) -> anyhow::Result<String> {
        let bytes = self.buffer.read_slice(35, 14);
        let value = String::from_utf8(bytes.to_vec())?;

        Ok(value)
    }
}

impl WValue for DeviceInfo {
    const VALUE: u16 = 0x000A;
}

impl WValueReadable for DeviceInfo {
    const REQUIRE_EXACT: bool = false;

    fn wave_buffer_mut(&mut self) -> &mut [u8] {
        self.buffer_mut()
    }
}

impl WValueWritable for DeviceInfo {
    fn wave_buffer(&self) -> &[u8] {
        self.buffer()
    }
}
