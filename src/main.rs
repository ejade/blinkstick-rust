use anyhow::Result;
use blinkstick::{BlinkStick, RgbColor};
use clap::{Parser, Subcommand};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "blinkstick")]
#[command(about = "Control BlinkStick devices", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Set color of BlinkStick device
    #[command(arg_required_else_help = true)]
    SetColor {
        /// Color name (red, green, blue, etc.) or hex value (#FF0000)
        color: String,
        
        /// LED index (0 for first LED, which is the default)
        #[arg(short, long, default_value = "0")]
        index: u8,
    },
    
    /// Pulse color on BlinkStick device
    #[command(arg_required_else_help = true)]
    Pulse {
        /// Color name (red, green, blue, etc.) or hex value (#FF0000)
        color: String,
        
        /// Duration of pulse in milliseconds
        #[arg(short, long, default_value = "1000")]
        duration: u32,
        
        /// Number of steps in the pulse
        #[arg(short, long, default_value = "20")]
        steps: u32,
    },
    
    /// List all connected BlinkStick devices
    List,
    
    /// Get info about BlinkStick device
    Info,
    
    /// Turn off BlinkStick (set color to black)
    Off,
    
    /// Add udev rule for BlinkStick devices on Linux
    AddUdevRule {
        /// Path to udev rules directory
        #[arg(short, long, default_value = "/etc/udev/rules.d")]
        path: PathBuf,
    },
}

fn parse_color(color_str: &str) -> Result<RgbColor> {
    // First try named color
    if let Some(color) = RgbColor::from_name(color_str) {
        return Ok(color);
    }
    
    // Then try hex color
    if let Some(color) = RgbColor::from_hex(color_str) {
        return Ok(color);
    }
    
    // Special case for random color
    if color_str.to_lowercase() == "random" {
        return Ok(RgbColor::random());
    }
    
    anyhow::bail!("Invalid color: {}", color_str)
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::SetColor { color, index } => {
            let blinkstick = BlinkStick::find_first()?;
            let color = parse_color(&color)?;
            
            if index == 0 {
                blinkstick.set_color(&color)?;
            } else {
                blinkstick.set_color_indexed(index, &color)?;
            }
            
            println!("Set color to RGB({}, {}, {}) at index {}", color.r, color.g, color.b, index);
        },
        
        Commands::Pulse { color, duration, steps } => {
            let blinkstick = BlinkStick::find_first()?;
            let color = parse_color(&color)?;
            
            println!("Pulsing RGB({}, {}, {}) for {}ms with {} steps", 
                color.r, color.g, color.b, duration, steps);
            
            blinkstick.pulse(&color, duration, steps)?;
        },
        
        Commands::List => {
            let devices = blinkstick::find_blinksticks()?;
            
            if devices.is_empty() {
                println!("No BlinkStick devices found");
                return Ok(());
            }
            
            println!("Found {} BlinkStick device(s):", devices.len());
            
            for (i, device) in devices.iter().enumerate() {
                let blinkstick = BlinkStick::open(device.clone())?;
                let serial = blinkstick.get_serial().unwrap_or_else(|_| "Unknown".to_string());
                
                println!("  {}. Serial: {}", i + 1, serial);
            }
        },
        
        Commands::Info => {
            let blinkstick = BlinkStick::find_first()?;
            let serial = blinkstick.get_serial().unwrap_or_else(|_| "Unknown".to_string());
            let color = blinkstick.get_color()?;
            
            println!("BlinkStick Information:");
            println!("  Serial: {}", serial);
            println!("  Current Color: RGB({}, {}, {})", color.r, color.g, color.b);
        },
        
        Commands::Off => {
            let blinkstick = BlinkStick::find_first()?;
            blinkstick.set_color(&RgbColor::new(0, 0, 0))?;
            println!("BlinkStick turned off");
        },
        
        Commands::AddUdevRule { path } => {
            if !cfg!(target_os = "linux") {
                println!("This command is only available on Linux");
                return Ok(());
            }
            
            let rule_path = path.join("85-blinkstick.rules");
            let rule = "SUBSYSTEM==\"usb\", ATTR{idVendor}==\"20a0\", ATTR{idProduct}==\"41e5\", MODE:=\"0666\"";
            
            let mut file = File::create(&rule_path)?;
            writeln!(file, "{}", rule)?;
            
            println!("Udev rule added to: {}", rule_path.display());
            println!("Reboot your computer or run 'sudo udevadm control --reload-rules && sudo udevadm trigger' for the changes to take effect");
        },
    }
    
    Ok(())
}
