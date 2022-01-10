# Changelog

Logs all version updates within the project.
## [0.3.1]

### Added
- A `carousel` function that uses all available leds on the BlinkStick device to transition between two specified Colors
### Changed
- Reintroduced the Drop trait
- Changed the way elapsed duration calculations occur to prevent overflow error

## [0.3.0]

### Added
- New "turn off" function family for a single, multiple or all leds

### Changed
- Functions now return Results, to give the user more flexibility during development

## [0.2.3]

### Added
- `transform_all_leds_color` function that transforms all leds to a single color

### Changed
- All `transform` family functions now utilizes the new `calculate_gradients` function
- The variable `steps` is now of size u16 instead of u64

### Removed
- Removed the short sleep between feature sets/gets, improves the animation performance

## [0.2.2]

### Added
- Implemented Default for BlinkStick
- Light resets to an off-state when creating a new BlinkStick object

### Changed
- Reduced the sleep timer between device gets/sets from 10 milliseconds to 2 milliseconds


## [0.2.1]

### Changed

- Added a 10-millisecond waiting period between each command when sending multiple commands to the blinkstick
## [0.2.0]

### Added 

- Added a function `get_random_color` to generate a random `Color`
- Added a function `get_color_vec` to return a proerly sized `Vec<Color>` depending on the BlinkStick device
- Added a function `set_all_leds_colors` to set different colors for every led on the BlinkStick device
- Added a function `blink_all_leds_color` to make all leds blink in a single `Color`
- Added a function `transform_multiple_leds_color` to transform the color of the specified leds into a single `Color`
- Added a function `pulse_all_leds_color` to make every single led pulse between their current color and a specified `Color`
- Added a new public `BlinkStick` struct variable `max_leds` that holds the number of leds available on the attached BlinkStick device
- A changelog document to track changes between versions

### Changed

- Made `get_all_led_colors` function public
- Made `get_led_color` function public
- Global usage of the now public `Color` struct
- Renamed functions to make space for future multi-color functions
- Various performance fixes

### Fixed

- Fixed a bug causing the project to not run as expected on Unix systems based on libusb

### Removed

- Removed various overlapping tests


[0.2.0]: https://github.com/Seltiix/blinkstick-rs/compare/HEAD...0.2.0
[0.2.1]: https://github.com/Seltiix/blinkstick-rs/compare/HEAD...0.2.1
[0.2.2]: https://github.com/Seltiix/blinkstick-rs/compare/HEAD...0.2.2
[0.2.3]: https://github.com/Seltiix/blinkstick-rs/compare/HEAD...0.2.3
[0.3.0]: https://github.com/Seltiix/blinkstick-rs/compare/HEAD...0.3.0
[0.3.0]: https://github.com/Seltiix/blinkstick-rs/compare/HEAD...0.3.1