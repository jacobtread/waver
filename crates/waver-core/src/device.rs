use rusb::{DeviceHandle, UsbContext};
use std::time::Duration;
use thiserror::Error;

const VENDOR_ID: u16 = 0x0FD9;
const PRODUCT_ID: u16 = 0x007D;

const BREQUEST_READ: u8 = 0x85;
const BREQUEST_WRITE: u8 = 0x05;

/// We use this index to bypass interfering with the kernel
/// interface routing
const WINDEX: u16 = 0x3303;

pub struct WaveXLRDevice {
    /// Handle to the underlying usb device
    handle: DeviceHandle<rusb::Context>,
}

pub trait WValue {
    const VALUE: u16;
}

pub trait WValueReadable: WValue + Default {
    /// Whether to require that the bytes read exactly matches
    /// the buffer size
    const REQUIRE_EXACT: bool;

    /// Get the underlying writable buffer for writing to
    fn wave_buffer_mut(&mut self) -> &mut [u8];
}

pub trait WValueWritable: WValue {
    /// Get the underlying buffer to write to the device
    fn wave_buffer(&self) -> &[u8];
}

#[derive(Debug, Error)]
pub enum WaveError {
    #[error(transparent)]
    Usb(#[from] rusb::Error),

    #[error("failed to open device handle")]
    OpenHandleFailed,

    #[error("expected {0} bytes but only got {1}")]
    InvalidResponseLength(usize, usize),
}

impl WaveError {
    pub fn is_disconnected(&self) -> bool {
        match self {
            WaveError::Usb(error) => matches!(
                error,
                rusb::Error::Io | rusb::Error::NoDevice | rusb::Error::Pipe
            ),
            _ => false,
        }
    }
}

impl WaveXLRDevice {
    /// Connect to the Wave XLR device
    pub fn connect() -> Result<Self, WaveError> {
        let context = rusb::Context::new()?;
        let handle = context
            .open_device_with_vid_pid(VENDOR_ID, PRODUCT_ID)
            .ok_or(WaveError::OpenHandleFailed)?;

        Ok(Self { handle })
    }

    fn ctrl_read(&mut self, w_value: u16, buffer: &mut [u8]) -> Result<usize, WaveError> {
        let request_type = rusb::request_type(
            rusb::Direction::In,
            rusb::RequestType::Class,
            rusb::Recipient::Interface,
        );

        let bytes_read = self.handle.read_control(
            request_type,
            BREQUEST_READ,
            w_value,
            WINDEX,
            buffer,
            Duration::from_millis(1000),
        )?;

        Ok(bytes_read)
    }

    fn ctrl_write(&mut self, w_value: u16, data: &[u8]) -> Result<(), WaveError> {
        // Construct wRequestType matching Python's RT_CLASS_OUT (0x21)
        let request_type = rusb::request_type(
            rusb::Direction::Out,
            rusb::RequestType::Class,
            rusb::Recipient::Interface,
        );

        self.handle.write_control(
            request_type,
            BREQUEST_WRITE,
            w_value,
            WINDEX,
            data,
            Duration::from_millis(1000),
        )?;

        Ok(())
    }

    /// Read a readable WValue from the device
    pub fn read<R: WValueReadable>(&mut self) -> Result<R, WaveError> {
        let mut readable = R::default();
        let buffer = readable.wave_buffer_mut();
        let bytes_read = self.ctrl_read(R::VALUE, buffer)?;
        if R::REQUIRE_EXACT && bytes_read != buffer.len() {
            return Err(WaveError::InvalidResponseLength(buffer.len(), bytes_read));
        }

        Ok(readable)
    }

    /// Write a writable WValue to the device
    pub fn write<W: WValueWritable>(&mut self, value: &W) -> Result<(), WaveError> {
        self.ctrl_write(W::VALUE, value.wave_buffer())?;
        Ok(())
    }
}
