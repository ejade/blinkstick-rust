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
    
    /// List all available color names
    ListColors,
    
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
        
        Commands::ListColors => {
            list_available_colors();
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

fn list_available_colors() {
    let colors = [
        "aliceblue", "antiquewhite", "aqua", "aquamarine", "azure",
        "beige", "bisque", "black", "blanchedalmond", "blue",
        "blueviolet", "brown", "burlywood", "cadetblue", "chartreuse",
        "chocolate", "coral", "cornflowerblue", "cornsilk", "crimson",
        "cyan", "darkblue", "darkcyan", "darkgoldenrod", "darkgray",
        "darkgreen", "darkgrey", "darkkhaki", "darkmagenta", "darkolivegreen",
        "darkorange", "darkorchid", "darkred", "darksalmon", "darkseagreen",
        "darkslateblue", "darkslategray", "darkslategrey", "darkturquoise", "darkviolet",
        "deeppink", "deepskyblue", "dimgray", "dimgrey", "dodgerblue",
        "firebrick", "floralwhite", "forestgreen", "fuchsia", "gainsboro",
        "ghostwhite", "gold", "goldenrod", "gray", "green",
        "greenyellow", "grey", "honeydew", "hotpink", "indianred",
        "indigo", "ivory", "khaki", "lavender", "lavenderblush",
        "lawngreen", "lemonchiffon", "lightblue", "lightcoral", "lightcyan",
        "lightgoldenrodyellow", "lightgray", "lightgreen", "lightgrey", "lightpink",
        "lightsalmon", "lightseagreen", "lightskyblue", "lightslategray", "lightslategrey",
        "lightsteelblue", "lightyellow", "lime", "limegreen", "linen",
        "magenta", "maroon", "mediumaquamarine", "mediumblue", "mediumorchid",
        "mediumpurple", "mediumseagreen", "mediumslateblue", "mediumspringgreen", "mediumturquoise",
        "mediumvioletred", "midnightblue", "mintcream", "mistyrose", "moccasin",
        "navajowhite", "navy", "oldlace", "olive", "olivedrab",
        "orange", "orangered", "orchid", "palegoldenrod", "palegreen",
        "paleturquoise", "palevioletred", "papayawhip", "peachpuff", "peru",
        "pink", "plum", "powderblue", "purple", "red",
        "rosybrown", "royalblue", "saddlebrown", "salmon", "sandybrown",
        "seagreen", "seashell", "sienna", "silver", "skyblue",
        "slateblue", "slategray", "slategrey", "snow", "springgreen",
        "steelblue", "tan", "teal", "thistle", "tomato",
        "turquoise", "violet", "wheat", "white", "whitesmoke",
        "yellow", "yellowgreen", "random"
    ];

    println!("Available colors:");
    
    // Calculate column formatting
    let num_colors = colors.len();
    let max_color_length = colors.iter().map(|c| c.len()).max().unwrap_or(10);
    let column_width = max_color_length + 2; // Add 2 for spacing
    
    // Determine number of columns based on terminal width (assume 80 chars by default)
    let terminal_width = 80;
    let num_columns = std::cmp::max(1, terminal_width / column_width);
    let num_rows = (num_colors + num_columns - 1) / num_columns; // Ceiling division
    
    // Print in columns
    for row in 0..num_rows {
        let mut line = String::new();
        
        for col in 0..num_columns {
            let index = col * num_rows + row;
            if index < num_colors {
                // Format each color with fixed width
                let color_display = format!("{:<width$}", colors[index], width=column_width);
                line.push_str(&color_display);
            }
        }
        
        println!("{}", line);
    }
}
