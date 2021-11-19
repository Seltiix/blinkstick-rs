# blinkstick-rs [![Version](https://img.shields.io/crates/v/blinkstick-rs.svg)](https://crates.io/crates/blinkstick-rs) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/seltiix/blinkstick-rs/blob/master/LICENSE.txt) [![Documentation](https://docs.rs/blinkstick-rs/badge.svg)](https://docs.rs/blinkstick-rs)

blinkstick-rs provides an interface to control any [BlinkStick device](https://www.blinkstick.com/) using Rust.

# Examples

Sets the color of the 0th led to red
```rust
use blinkstick::BlinkStick;

let blinkstick = BlinkStick::new(); 
blinkstick.set_color(0, 50, 0, 0);
```

Sets the color of 0th, 2nd, 4th and 6th led to green.
```rust
use blinkstick::BlinkStick;

let blinkstick = BlinkStick::new();
blinkstick.set_unified_color(&vec![0, 2, 4, 6], 0, 50, 0);
 ```

Turns every led blue
```rust
use blinkstick::BlinkStick;
 
let blinkstick = BlinkStick::new();
blinkstick.set_all_colors(0, 0, 50);
```

Makes the 0th led blink 5 times, once every second, with a purple glow
```rust
use blinkstick::BlinkStick;

let blinkstick = BlinkStick::new();
blinkstick.blink_color(0, std::time::Duration::from_secs(1), 5, 25, 0, 25);
```

Makes the 1st, 3rd, 5th led blink 2 times, once every 200 miliseconds, with a yellow glow
```rust
use blinkstick::BlinkStick;

let blinkstick = BlinkStick::new();
blinkstick.blink_unified_color(&vec![1, 3, 5], std::time::Duration::from_millis(200), 2, 50, 50, 0);
```

Makes the 2nd led, pulse from an off state, to a blue glow, and then return back again to the off state with a two second animation time
```rust
use blinkstick::BlinkStick;
 
let blinkstick = BlinkStick::new();
blinkstick.pulse_color(2, std::time::Duration::from_secs(2), 20, 0, 0, 155);
```

Makes the 1st led transform from a red color into a green color over a period of five seconds, with 50 color updates.
```rust
use blinkstick::BlinkStick;

let blinkstick = BlinkStick::new();
blinkstick.set_color(1, 50, 0, 0);
blinkstick.transform_color(1, std::time::Duration::from_secs(5), 50, 0, 50, 0);
```