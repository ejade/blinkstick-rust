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
        // Standard CSS/HTML color names
        match name.to_lowercase().as_str() {
            "aliceblue" => Some(Self::new(240, 248, 255)),
            "antiquewhite" => Some(Self::new(250, 235, 215)),
            "aqua" | "cyan" => Some(Self::new(0, 255, 255)),
            "aquamarine" => Some(Self::new(127, 255, 212)),
            "azure" => Some(Self::new(240, 255, 255)),
            "beige" => Some(Self::new(245, 245, 220)),
            "bisque" => Some(Self::new(255, 228, 196)),
            "black" | "off" => Some(Self::new(0, 0, 0)),
            "blanchedalmond" => Some(Self::new(255, 235, 205)),
            "blue" => Some(Self::new(0, 0, 255)),
            "blueviolet" => Some(Self::new(138, 43, 226)),
            "brown" => Some(Self::new(165, 42, 42)),
            "burlywood" => Some(Self::new(222, 184, 135)),
            "cadetblue" => Some(Self::new(95, 158, 160)),
            "chartreuse" => Some(Self::new(127, 255, 0)),
            "chocolate" => Some(Self::new(210, 105, 30)),
            "coral" => Some(Self::new(255, 127, 80)),
            "cornflowerblue" => Some(Self::new(100, 149, 237)),
            "cornsilk" => Some(Self::new(255, 248, 220)),
            "crimson" => Some(Self::new(220, 20, 60)),
            "darkblue" => Some(Self::new(0, 0, 139)),
            "darkcyan" => Some(Self::new(0, 139, 139)),
            "darkgoldenrod" => Some(Self::new(184, 134, 11)),
            "darkgray" | "darkgrey" => Some(Self::new(169, 169, 169)),
            "darkgreen" => Some(Self::new(0, 100, 0)),
            "darkkhaki" => Some(Self::new(189, 183, 107)),
            "darkmagenta" => Some(Self::new(139, 0, 139)),
            "darkolivegreen" => Some(Self::new(85, 107, 47)),
            "darkorange" => Some(Self::new(255, 140, 0)),
            "darkorchid" => Some(Self::new(153, 50, 204)),
            "darkred" => Some(Self::new(139, 0, 0)),
            "darksalmon" => Some(Self::new(233, 150, 122)),
            "darkseagreen" => Some(Self::new(143, 188, 143)),
            "darkslateblue" => Some(Self::new(72, 61, 139)),
            "darkslategray" | "darkslategrey" => Some(Self::new(47, 79, 79)),
            "darkturquoise" => Some(Self::new(0, 206, 209)),
            "darkviolet" => Some(Self::new(148, 0, 211)),
            "deeppink" => Some(Self::new(255, 20, 147)),
            "deepskyblue" => Some(Self::new(0, 191, 255)),
            "dimgray" | "dimgrey" => Some(Self::new(105, 105, 105)),
            "dodgerblue" => Some(Self::new(30, 144, 255)),
            "firebrick" => Some(Self::new(178, 34, 34)),
            "floralwhite" => Some(Self::new(255, 250, 240)),
            "forestgreen" => Some(Self::new(34, 139, 34)),
            "fuchsia" | "magenta" => Some(Self::new(255, 0, 255)),
            "gainsboro" => Some(Self::new(220, 220, 220)),
            "ghostwhite" => Some(Self::new(248, 248, 255)),
            "gold" => Some(Self::new(255, 215, 0)),
            "goldenrod" => Some(Self::new(218, 165, 32)),
            "gray" | "grey" => Some(Self::new(128, 128, 128)),
            "green" => Some(Self::new(0, 128, 0)),
            "greenyellow" => Some(Self::new(173, 255, 47)),
            "honeydew" => Some(Self::new(240, 255, 240)),
            "hotpink" => Some(Self::new(255, 105, 180)),
            "indianred" => Some(Self::new(205, 92, 92)),
            "indigo" => Some(Self::new(75, 0, 130)),
            "ivory" => Some(Self::new(255, 255, 240)),
            "khaki" => Some(Self::new(240, 230, 140)),
            "lavender" => Some(Self::new(230, 230, 250)),
            "lavenderblush" => Some(Self::new(255, 240, 245)),
            "lawngreen" => Some(Self::new(124, 252, 0)),
            "lemonchiffon" => Some(Self::new(255, 250, 205)),
            "lightblue" => Some(Self::new(173, 216, 230)),
            "lightcoral" => Some(Self::new(240, 128, 128)),
            "lightcyan" => Some(Self::new(224, 255, 255)),
            "lightgoldenrodyellow" => Some(Self::new(250, 250, 210)),
            "lightgray" | "lightgrey" => Some(Self::new(211, 211, 211)),
            "lightgreen" => Some(Self::new(144, 238, 144)),
            "lightpink" => Some(Self::new(255, 182, 193)),
            "lightsalmon" => Some(Self::new(255, 160, 122)),
            "lightseagreen" => Some(Self::new(32, 178, 170)),
            "lightskyblue" => Some(Self::new(135, 206, 250)),
            "lightslategray" | "lightslategrey" => Some(Self::new(119, 136, 153)),
            "lightsteelblue" => Some(Self::new(176, 196, 222)),
            "lightyellow" => Some(Self::new(255, 255, 224)),
            "lime" => Some(Self::new(0, 255, 0)),
            "limegreen" => Some(Self::new(50, 205, 50)),
            "linen" => Some(Self::new(250, 240, 230)),
            "maroon" => Some(Self::new(128, 0, 0)),
            "mediumaquamarine" => Some(Self::new(102, 205, 170)),
            "mediumblue" => Some(Self::new(0, 0, 205)),
            "mediumorchid" => Some(Self::new(186, 85, 211)),
            "mediumpurple" => Some(Self::new(147, 112, 216)),
            "mediumseagreen" => Some(Self::new(60, 179, 113)),
            "mediumslateblue" => Some(Self::new(123, 104, 238)),
            "mediumspringgreen" => Some(Self::new(0, 250, 154)),
            "mediumturquoise" => Some(Self::new(72, 209, 204)),
            "mediumvioletred" => Some(Self::new(199, 21, 133)),
            "midnightblue" => Some(Self::new(25, 25, 112)),
            "mintcream" => Some(Self::new(245, 255, 250)),
            "mistyrose" => Some(Self::new(255, 228, 225)),
            "moccasin" => Some(Self::new(255, 228, 181)),
            "navajowhite" => Some(Self::new(255, 222, 173)),
            "navy" => Some(Self::new(0, 0, 128)),
            "oldlace" => Some(Self::new(253, 245, 230)),
            "olive" => Some(Self::new(128, 128, 0)),
            "olivedrab" => Some(Self::new(107, 142, 35)),
            "orange" => Some(Self::new(255, 165, 0)),
            "orangered" => Some(Self::new(255, 69, 0)),
            "orchid" => Some(Self::new(218, 112, 214)),
            "palegoldenrod" => Some(Self::new(238, 232, 170)),
            "palegreen" => Some(Self::new(152, 251, 152)),
            "paleturquoise" => Some(Self::new(175, 238, 238)),
            "palevioletred" => Some(Self::new(216, 112, 147)),
            "papayawhip" => Some(Self::new(255, 239, 213)),
            "peachpuff" => Some(Self::new(255, 218, 185)),
            "peru" => Some(Self::new(205, 133, 63)),
            "pink" => Some(Self::new(255, 192, 203)),
            "plum" => Some(Self::new(221, 160, 221)),
            "powderblue" => Some(Self::new(176, 224, 230)),
            "purple" => Some(Self::new(128, 0, 128)),
            "red" => Some(Self::new(255, 0, 0)),
            "rosybrown" => Some(Self::new(188, 143, 143)),
            "royalblue" => Some(Self::new(65, 105, 225)),
            "saddlebrown" => Some(Self::new(139, 69, 19)),
            "salmon" => Some(Self::new(250, 128, 114)),
            "sandybrown" => Some(Self::new(244, 164, 96)),
            "seagreen" => Some(Self::new(46, 139, 87)),
            "seashell" => Some(Self::new(255, 245, 238)),
            "sienna" => Some(Self::new(160, 82, 45)),
            "silver" => Some(Self::new(192, 192, 192)),
            "skyblue" => Some(Self::new(135, 206, 235)),
            "slateblue" => Some(Self::new(106, 90, 205)),
            "slategray" | "slategrey" => Some(Self::new(112, 128, 144)),
            "snow" => Some(Self::new(255, 250, 250)),
            "springgreen" => Some(Self::new(0, 255, 127)),
            "steelblue" => Some(Self::new(70, 130, 180)),
            "tan" => Some(Self::new(210, 180, 140)),
            "teal" => Some(Self::new(0, 128, 128)),
            "thistle" => Some(Self::new(216, 191, 216)),
            "tomato" => Some(Self::new(255, 99, 71)),
            "turquoise" => Some(Self::new(64, 224, 208)),
            "violet" => Some(Self::new(238, 130, 238)),
            "wheat" => Some(Self::new(245, 222, 179)),
            "white" => Some(Self::new(255, 255, 255)),
            "whitesmoke" => Some(Self::new(245, 245, 245)),
            "yellow" => Some(Self::new(255, 255, 0)),
            "yellowgreen" => Some(Self::new(154, 205, 50)),
            "random" => Some(Self::random()),
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
        let data = [REPORT_ID_1, color.r, color.g, color.b];
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
            data.push(color.r);
            data.push(color.g);
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