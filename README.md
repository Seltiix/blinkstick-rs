# blinkstick-rs [![Version](https://img.shields.io/crates/v/blinkstick-rs.svg)](https://crates.io/crates/blinkstick-rs) [![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/seltiix/blinkstick-rs/blob/master/LICENSE.txt) [![Documentation](https://docs.rs/blinkstick-rs/badge.svg)](https://docs.rs/blinkstick-rs)

blinkstick-rs provides an interface to control any [BlinkStick device](https://www.blinkstick.com/) using Rust.

# About
# Examples


```rust
    use blinkstick::BlinkStick;
    let blinkstick = BlinkStick::new(); 
    blinkstick.set_color(0, 50, 0, 0);
```