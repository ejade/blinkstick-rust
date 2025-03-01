# BlinkStick Rust
A Rust library and command-line tool for controlling BlinkStick USB LED devices.
## What is BlinkStick?
BlinkStick is a smart USB LED controller that can be controlled programmatically. Visit blinkstick.com for more information about the hardware.

This has only been tested on Linux ubuntu 24.04.2 LTS. With blinkstick square.

## Installation

#### Clone the repository
```bash
git clone https://github.com/ejade/blinkstick-rust
cd blinkstick-rust
```

#### Build and install
```bash
cargo install --path .
```
### Linux permissions
On Linux, you need permission to access USB devices. You can add a udev rule using the built-in command:
```bash
sudo blinkstick add-udev-rule
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## Usage

#### Set LED color
```bash
# Set to a named color
blinkstick set-color red

# Set to a hex color
blinkstick set-color "#00FF00"

# Set a specific LED (for BlinkStick Pro)
blinkstick set-color blue --index 2

# Set to a random color
blinkstick set-color random
```

#### Pulse effect
```bash
blinkstick pulse red --duration 1000 --steps 20
```
#### List connected devices
```bash
blinkstick list
```
#### Get device information
```bash
blinkstick info
```
#### Turn off LED
```bash
blinkstick off
```

## License
This project is licensed under the MIT License. See the LICENSE file for details.

## Contributing
Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

