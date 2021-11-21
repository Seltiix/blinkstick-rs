# Changelog

Logs all version updates within the project.

## [Unreleased-0.2.0]

### Added 

- Added a function `get_random_color` to generate a random `Color`
- Added a function `get_color_vec` to return a proerly sized `Vec<Color>` depending on the BlinkStick device
- Added a function `set_all_colors` to set different colors for every led on the BlinkStick device
- Added a function `blink_all_color` to make all leds blink in a single `Color`
- Added a function `transform_unified_color` to transform the color of the specified leds into a single `Color`
- Added a new public `BlinkStick` struct variable `max_leds` that holds the number of leds available on the attached BlinkStick device
- A changelog document to track changes between versions

### Changed

- Global usage of the now public `Color` struct
- Renamed functions to make space for future multi-color functions
- Various performance fixes

### Fixed

- Fixed a bug causing the project to not run as expected on Unix systems based on libusb

### Removed

- Removed various overlapping tests


[Unreleased-0.2.0]: https://github.com/Seltiix/blinkstick-rs/compare/HEAD...0.2.0