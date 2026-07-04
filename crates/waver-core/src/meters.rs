use crate::device::{WValue, WValueReadable, WValueWritable};

const BUFFER_LENGTH: usize = 10;

struct MetersBuffer {
    buffer: [u8; BUFFER_LENGTH],
}

impl Default for MetersBuffer {
    fn default() -> Self {
        Self {
            buffer: [0u8; BUFFER_LENGTH],
        }
    }
}

impl MetersBuffer {
    fn buffer_mut(&mut self) -> &mut [u8] {
        &mut self.buffer
    }

    fn buffer(&self) -> &[u8] {
        &self.buffer
    }

    fn read_u32(&self, offset: usize) -> u32 {
        debug_assert!(
            offset < BUFFER_LENGTH && offset + 3 < BUFFER_LENGTH,
            "offset must be within bounds of config buffer"
        );

        let mut bytes = [0u8; 4];
        let bytes_ref = &self.buffer[offset..=offset + 1];
        bytes.copy_from_slice(bytes_ref);
        u32::from_le_bytes(bytes)
    }
}

#[derive(Default)]
pub struct Meters {
    buffer: MetersBuffer,
}

impl Meters {
    fn buffer_mut(&mut self) -> &mut [u8] {
        self.buffer.buffer_mut()
    }

    fn buffer(&self) -> &[u8] {
        self.buffer.buffer()
    }

    pub fn read_left(&self) -> u32 {
        self.buffer.read_u32(0)
    }

    pub fn read_right(&self) -> u32 {
        self.buffer.read_u32(1)
    }
}

impl WValue for Meters {
    const VALUE: u16 = 0x0001;
}

impl WValueReadable for Meters {
    const REQUIRE_EXACT: bool = false;

    fn wave_buffer_mut(&mut self) -> &mut [u8] {
        self.buffer_mut()
    }
}

impl WValueWritable for Meters {
    fn wave_buffer(&self) -> &[u8] {
        self.buffer()
    }
}
