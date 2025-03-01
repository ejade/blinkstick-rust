//! BlinkStick Rust Library
//!
//! A Rust interface to control BlinkStick devices connected to the computer.
//! BlinkStick is a smart USB LED pixel. More info: https://www.blinkstick.com

use anyhow::{anyhow, Result};
use rusb::{Context, Device, DeviceHandle, UsbContext};
use std::time::Duration;
use thiserror::Error;

// BlinkStick USB identifiers
const BLINKSTICK_VENDOR_ID: u16 = 0x20A0;
const BLINKSTICK_PRODUCT_ID: u16 = 0x41E5;

// BlinkStick report IDs
const REPORT_ID_1: u8 = 1; // First LED for BlinkStick
const REPORT_ID_2: u8 = 2; // 8 LEDs for BlinkStick Pro
const REPORT_ID_3: u8 = 3; // 64+ LEDs for BlinkStick Pro
//const REPORT_ID_4: u8 = 4; // Inverse LED control

#[derive(Debug, Error)]
pub enum BlinkStickError {
    #[error("USB error: {0}")]
    UsbError(#[from] rusb::Error),
    
    #[error("No BlinkStick devices found")]
    NoDeviceFound,
    
    #[error("Failed to get device descriptor")]
    DeviceDescriptorError,
    
    #[error("Failed to open device")]
    OpenDeviceError,
    
    #[error("Failed to claim interface")]
    ClaimInterfaceError,
    
    #[error("Failed to set configuration")]
    SetConfigurationError,
    
    #[error("Failed to send control transfer")]
    ControlTransferError,
}

#[derive(Debug, Clone)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
    
    pub fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        Self {
            r: rng.gen(),
            g: rng.gen(),
            b: rng.gen(),
        }
    }
    
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "red" => Some(Self::new(0, 255, 0)),
            "green" => Some(Self::new(255, 0, 0)),
            "blue" => Some(Self::new(0, 0, 255)),
            "yellow" => Some(Self::new(255, 255, 0)),
            "cyan" => Some(Self::new(128, 0, 128)),
            "purple" => Some(Self::new(0, 255, 255)),
            "white" => Some(Self::new(255, 255, 255)),
            "black" | "off" => Some(Self::new(0, 0, 0)),
            // Add more named colors as needed
            _ => None,
        }
    }
    
    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        
        if hex.len() == 6 {
            if let Ok(val) = u32::from_str_radix(hex, 16) {
                return Some(Self {
                    r: ((val >> 16) & 0xFF) as u8,
                    g: ((val >> 8) & 0xFF) as u8,
                    b: (val & 0xFF) as u8,
                });
            }
        }
        
        None
    }
}

pub struct BlinkStick {
    handle: DeviceHandle<Context>,
}

impl BlinkStick {
    /// Find all connected BlinkStick devices
    pub fn find_all() -> Result<Vec<Device<Context>>> {
        let context = Context::new()?;
        let devices = context.devices()?;
        let mut result = Vec::new();
        
        for device in devices.iter() {
            let device_desc = device.device_descriptor().map_err(|_| BlinkStickError::DeviceDescriptorError)?;
            
            if device_desc.vendor_id() == BLINKSTICK_VENDOR_ID && 
               device_desc.product_id() == BLINKSTICK_PRODUCT_ID {
                result.push(device);
            }
        }
        
        Ok(result)
    }
    
    /// Find the first available BlinkStick device
    pub fn find_first() -> Result<Self> {
        let devices = Self::find_all()?;
        
        if devices.is_empty() {
            return Err(BlinkStickError::NoDeviceFound.into());
        }
        
        Self::open(devices[0].clone())
    }
    
    /// Open a specific BlinkStick device
    pub fn open(device: Device<Context>) -> Result<Self> {
        let handle = device.open().map_err(|_| BlinkStickError::OpenDeviceError)?;
        
        if handle.kernel_driver_active(0)? {
            handle.detach_kernel_driver(0)?;
        }
        
        handle.set_active_configuration(1).map_err(|_| BlinkStickError::SetConfigurationError)?;
        handle.claim_interface(0).map_err(|_| BlinkStickError::ClaimInterfaceError)?;
        
        Ok(Self { handle })
    }
    
    /// Set the color of the first LED
    pub fn set_color(&self, color: &RgbColor) -> Result<()> {
        let data = [REPORT_ID_1, color.g, color.r, color.b];
        self.send_feature_report(&data)
    }
    
    /// Set the color of a specific LED for BlinkStick Pro
    pub fn set_color_indexed(&self, index: u8, color: &RgbColor) -> Result<()> {
        if index == 0 {
            // For the first LED, use report ID 1
            return self.set_color(color);
        }
        
        // For other LEDs, use report ID 2
        let data = [REPORT_ID_2, index, color.g, color.r, color.b];
        self.send_feature_report(&data)
    }
    
    /// Set colors for multiple LEDs at once
    pub fn set_colors(&self, channel: u8, leds: &[RgbColor]) -> Result<()> {
        if leds.is_empty() {
            return Ok(());
        }
        
        if leds.len() == 1 {
            return self.set_color(&leds[0]);
        }
        
        // For multiple LEDs, use report ID 3
        let mut data = vec![REPORT_ID_3, channel, 0, leds.len() as u8];
        
        for color in leds {
            data.push(color.g);
            data.push(color.r);
            data.push(color.b);
        }
        
        self.send_feature_report(&data)
    }
    
    /// Create a pulse effect
    pub fn pulse(&self, color: &RgbColor, duration_ms: u32, steps: u32) -> Result<()> {
        let step_delay = Duration::from_millis((duration_ms / steps) as u64);
        
        // Fade in
        for i in 0..steps {
            let factor = i as f32 / steps as f32;
            let fade_color = RgbColor {
                r: (color.r as f32 * factor) as u8,
                g: (color.g as f32 * factor) as u8,
                b: (color.b as f32 * factor) as u8,
            };
            self.set_color(&fade_color)?;
            std::thread::sleep(step_delay);
        }
        
        // Fade out
        for i in 0..steps {
            let factor = 1.0 - (i as f32 / steps as f32);
            let fade_color = RgbColor {
                r: (color.r as f32 * factor) as u8,
                g: (color.g as f32 * factor) as u8,
                b: (color.b as f32 * factor) as u8,
            };
            self.set_color(&fade_color)?;
            std::thread::sleep(step_delay);
        }
        
        Ok(())
    }
    
    /// Get the current color of the first LED
    pub fn get_color(&self) -> Result<RgbColor> {
        let mut data = [0u8; 4];
        data[0] = REPORT_ID_1;
        
        self.get_feature_report(&mut data)?;
        
        Ok(RgbColor {
            r: data[2],
            g: data[1],
            b: data[3],
        })
    }
    
    /// Get the device serial number
    pub fn get_serial(&self) -> Result<String> {
        let mut data = [0u8; 256];
        
        let len = self.handle.read_control(
            rusb::request_type(rusb::Direction::In, rusb::RequestType::Standard, rusb::Recipient::Device),
            rusb::constants::LIBUSB_REQUEST_GET_DESCRIPTOR, 
            (rusb::constants::LIBUSB_DT_STRING as u16) << 8 | 3,
            0, 
            &mut data, 
            Duration::from_secs(1)
        )?;
        
        if len <= 2 {
            return Err(anyhow!("Failed to get serial number"));
        }
        
        // Convert UTF-16LE to String
        let utf16_chars: Vec<u16> = data[2..len]
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .collect();
        
        String::from_utf16(&utf16_chars).map_err(|e| anyhow!("Failed to decode serial number: {}", e))
    }
    
    /// Helper function to send feature reports
    fn send_feature_report(&self, data: &[u8]) -> Result<()> {
        self.handle.write_control(
            rusb::request_type(rusb::Direction::Out, rusb::RequestType::Class, rusb::Recipient::Interface),
            0x09, // SET_REPORT
            0x0300 | data[0] as u16, // HID_REPORT_TYPE_FEATURE | report_id
            0,
            data,
            Duration::from_secs(1)
        ).map_err(|_| BlinkStickError::ControlTransferError)?;
        
        Ok(())
    }
    
    /// Helper function to get feature reports
    fn get_feature_report(&self, data: &mut [u8]) -> Result<()> {
        let report_id = data[0];
        
        self.handle.read_control(
            rusb::request_type(rusb::Direction::In, rusb::RequestType::Class, rusb::Recipient::Interface),
            0x01, // GET_REPORT
            0x0300 | report_id as u16, // HID_REPORT_TYPE_FEATURE | report_id
            0,
            data,
            Duration::from_secs(1)
        ).map_err(|_| BlinkStickError::ControlTransferError)?;
        
        Ok(())
    }
}

impl Drop for BlinkStick {
    fn drop(&mut self) {
        let _ = self.handle.release_interface(0);
    }
}

/// Helper function to retrieve all connected BlinkStick devices
pub fn find_blinksticks() -> Result<Vec<Device<Context>>> {
    BlinkStick::find_all()
}

/// Helper function to find the first available BlinkStick
pub fn find_first_blinkstick() -> Result<BlinkStick> {
    BlinkStick::find_first()
} 