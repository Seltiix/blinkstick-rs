# blinkstick-rs [![Version](https://img.shields.io/crates/v/blinkstick-rs.svg)](https://crates.io/crates/blinkstick-rs) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/seltiix/blinkstick-rs/blob/master/LICENSE.txt) [![Documentation](https://docs.rs/blinkstick-rs/badge.svg)](https://docs.rs/blinkstick-rs)

blinkstick-rs provides an interface to control any [BlinkStick device](https://www.blinkstick.com/) using Rust.

# Examples
:exclamation: For the non-published updates, please refer to function documentation for the latest examples.

Sets the color of a single led to red
```rust
use blinkstick_rs::{BlinkStick, Color};

let blinkstick = BlinkStick::new().unwrap();

blinkstick.set_led_color(0, Color {r: 50, g: 0, b: 0});
```

Sets a random color to every led on the BlinkStick device
```rust
use blinkstick_rs::{BlinkStick, Color};

let blinkstick = BlinkStick::new().unwrap();

let mut colors: Vec<Color> = blinkstick.get_color_vec();
for led in 0..blinkstick.max_leds as usize {
   colors[led] = BlinkStick::get_random_color();
}
blinkstick.set_all_leds_colors(&colors).unwrap();
```

Makes the 1st, 3rd, 5th LED blink 2 times, once every 200 milliseconds, with a yellow glow
```rust
use blinkstick_rs::{BlinkStick, Color};

let blinkstick = BlinkStick::new().unwrap();

blinkstick.blink_multiple_leds_color(&vec![1, 3, 5], std::time::Duration::from_millis(200), 2, Color {r: 50, g: 50, b: 0}).unwrap();
```

Makes every led pulse between being turned off and a green color
```rust
use blinkstick_rs::{BlinkStick, Color};

let blinkstick = BlinkStick::new().unwrap();

let color = Color {r: 0, g: 25, b: 0};
blinkstick.pulse_all_leds_color(std::time::Duration::from_secs(2), 50, Color {r: 0, g: 25, b: 0}).unwrap();
```

Makes the first led transform from a red color into a green color over a period of five seconds, with 50 color updates.
```rust
use blinkstick_rs::{BlinkStick, Color};

let blinkstick = BlinkStick::new().unwrap();

blinkstick.set_led_color(1, Color {r: 50, g: 0, b: 0}).unwrap();
blinkstick.transform_led_color(1, std::time::Duration::from_secs(5), 50, Color {r: 0, g: 50, b: 0}).unwrap();
```

# Running tests
Tests are only runnable when a BlinkStick device is plugged in. Furthermore, tests should be run using `cargo test -- --test-threads=1` or they might fail.

# Dependencies
## Linux
- libusb - Can be installed using for example "apt install usbutils" or "yum install usbutils"