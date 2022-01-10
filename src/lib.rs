//! This crate provides a rust toolkit for interacting with the BlinkStick device.
//! The implementation should support all types of BlinkStick devices. It was however
//! implemented and tested using a BlinkStick Square. If a BlinkStick device acts incorrectly, please contact me.
//! Requires libusb when using blinkstick-rs on Linux machines, check README for more information.

use crate::FeatureErrorType::{Get, Send};
use rand::Rng;
use std::fmt::Formatter;
use std::ops::{Div, Sub};
use std::{time::Duration, time::Instant};

extern crate hidapi;

const VENDOR_ID: u16 = 0x20a0;
const PRODUCT_ID: u16 = 0x41e5;

const REPORT_ARRAY_BYTES: usize = 100;

#[derive(Debug)]
pub struct FeatureError {
    pub kind: FeatureErrorType,
}

#[derive(Debug)]
pub enum FeatureErrorType {
    Get,
    Send,
}

impl std::fmt::Display for FeatureError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Could not communicate feature from/to BlinkStick")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}
const COLOR_OFF: Color = Color { r: 0, g: 0, b: 0 };

pub struct BlinkStick {
    device: hidapi::HidDevice,
    pub max_leds: u8,
    report_length: usize,
}

impl Drop for BlinkStick {
    fn drop(&mut self) {
        self.set_all_leds_color(COLOR_OFF).unwrap()
    }
}

impl Default for BlinkStick {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl BlinkStick {
    /// Opens communication with a `BlinkStick Device`
    /// # Panics
    /// When there is no connected BlinkStick device, the call to new will panic.
    pub fn new() -> Result<BlinkStick, FeatureError> {
        let api = hidapi::HidApi::new().expect("Could not create a hid api");

        let device = api.open(VENDOR_ID, PRODUCT_ID);

        let device = match device {
            Ok(device) => device,
            Err(error) => panic!("Problem connecting to device: {:?}", error),
        };

        // Determines the number of leds for a device. The BlinkStick Flex has 32 leds with 3 channels, which is the maximum of any device.
        // 32 * 3 + 2 = 98 bytes
        let mut buf: [u8; REPORT_ARRAY_BYTES] = [0; REPORT_ARRAY_BYTES];
        buf[0] = 0x6;
        let bytes_read = device.get_feature_report(&mut buf).unwrap();

        // First two bytes are meta information
        let max_leds = ((bytes_read - 2) / 3) as u8;
        let report_length = ((max_leds * 3) + 2).into();

        let blinkstick = BlinkStick {
            device,
            max_leds,
            report_length,
        };

        // If the light is already on, we want to reset it before giving the user a way to interact with it.
        blinkstick.set_all_leds_color(COLOR_OFF)?;

        Ok(blinkstick)
    }

    /// Turns off a single led
    ///
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    pub fn turn_off_led(&self, led: u8) -> Result<(), FeatureError> {
        self.set_led_color(led, COLOR_OFF)
    }

    /// Turns off multiple leds
    ///
    /// # Arguments
    /// * `leds` - Zero-indexed led numbers (within bounds for the BlinkStick product)
    pub fn turn_off_multiple_leds(&self, leds: &[u8]) -> Result<(), FeatureError> {
        self.set_multiple_leds_color(leds, COLOR_OFF)
    }

    /// Turns off all leds
    pub fn turn_off_all_leds(&self) -> Result<(), FeatureError> {
        self.set_all_leds_color(COLOR_OFF)
    }

    /// Generates a random color
    ///
    /// # Example
    /// Returns a random `Color`
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new().unwrap();
    ///
    /// let color = BlinkStick::get_random_color();
    /// ```
    pub fn get_random_color() -> Color {
        let mut rng = rand::thread_rng();

        Color {
            r: rng.gen_range(0..255),
            g: rng.gen_range(0..255),
            b: rng.gen_range(0..255),
        }
    }

    /// Returns a Vector with an appropriate vector length for the plugged in BlinkStick device
    ///
    /// # Example
    /// Returns a `Color` vector
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new().unwrap();
    ///
    /// let colors = blinkstick.get_color_vec();
    /// ```
    pub fn get_color_vec(&self) -> Vec<Color> {
        vec![Color { r: 0, g: 0, b: 0 }; self.max_leds as usize]
    }

    /// Sets the RGB color of a single led
    ///
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Panics
    /// The call to `set_led_color` will panic if the specified `led` is out of bounds for the connected BlinkStick device.
    ///
    /// # Example
    /// Sets the color of the 0th led to red
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new().unwrap();
    ///
    /// blinkstick.set_led_color(0, Color {r: 50, g: 0, b: 0}).unwrap();
    ///
    /// assert_eq!(blinkstick.get_led_color(0).unwrap(), Color {r: 50, g: 0, b: 0});
    /// ```
    pub fn set_led_color(&self, led: u8, color: Color) -> Result<(), FeatureError> {
        if led >= self.max_leds {
            panic!("Led {} is out of bounds for Blinkstick device", led)
        }

        self.send_feature_to_blinkstick(&[0x5, 0, led, color.r, color.g, color.b])
    }

    /// Sets the RGB color of one or more leds to a single color
    ///
    /// # Arguments
    /// * `leds` - A vector of zero-indexed led numbers (within bounds for the BlinkStick product)
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Panics
    /// The call to set_multiple_leds_color will panic if any of the specified `leds` is out of bounds for the BlinkStick device.
    ///
    /// # Example
    /// Sets the color of 0th, 2nd, 4th and 6th led to green.
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new().unwrap();
    /// blinkstick.set_multiple_leds_color(&vec![0, 2, 4, 6], Color {r: 0, g: 50, b: 0}).unwrap();
    /// ```
    pub fn set_multiple_leds_color(&self, leds: &[u8], color: Color) -> Result<(), FeatureError> {
        let mut data_vec: [u8; REPORT_ARRAY_BYTES] = [0; REPORT_ARRAY_BYTES];
        data_vec[0] = 0x6;

        for led in leds {
            let led_offset: usize = ((led * 3) + 2).into();

            if led_offset >= ((self.max_leds * 3) + 2).into() {
                panic!(
                    "BlinkStick device does not contain led {}. Valid leds are 0-{} (zero-indexed)",
                    led,
                    self.max_leds - 1
                );
            }

            data_vec[led_offset] = color.g;
            data_vec[led_offset + 1] = color.r;
            data_vec[led_offset + 2] = color.b;
        }

        self.send_feature_to_blinkstick(&data_vec[0..self.report_length])
    }

    /// Sets the same color for all leds available on the BlinkStick device
    ///
    /// # Arguments
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Example
    /// Turns every led blue
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new().unwrap();
    /// blinkstick.set_all_leds_color(Color {r: 0, g: 0, b: 50}).unwrap();
    /// ```
    pub fn set_all_leds_color(&self, color: Color) -> Result<(), FeatureError> {
        let leds: Vec<u8> = (0..self.max_leds).collect();
        self.set_multiple_leds_color(&leds, color)
    }

    /// Sets a different color for every led available on the BlinkStick device
    ///
    /// # Arguments
    /// * `colors` - A vector of equal length to the number of leds available on the device.
    ///
    /// # Panics
    /// The call to `blink_led_color` will panic if the length of the color vector is greater then the number of available leds
    ///
    /// # Example
    /// Sets a different color for each led on the device
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new().unwrap();
    ///
    /// let mut colors: Vec<Color> = blinkstick.get_color_vec();
    ///
    /// for led in 0..blinkstick.max_leds as usize {
    ///    colors[led] = BlinkStick::get_random_color();
    /// }
    ///
    /// blinkstick.set_all_leds_colors(&colors).unwrap();
    /// ```
    pub fn set_all_leds_colors(&self, colors: &[Color]) -> Result<(), FeatureError> {
        let mut data_vec: [u8; REPORT_ARRAY_BYTES] = [0; REPORT_ARRAY_BYTES];
        data_vec[0] = 0x6;

        for (led_index, led_color) in colors.iter().enumerate().take(self.max_leds as usize) {
            //for led in 0..self.max_leds as usize {
            let led_offset = (led_index * 3) + 2;

            if led_offset >= ((self.max_leds * 3) + 2).into() {
                panic!(
                    "BlinkStick device does not contain led {}. Valid leds are 0-{} (zero-indexed)",
                    led_index,
                    self.max_leds - 1
                );
            }

            data_vec[led_offset] = led_color.g;
            data_vec[led_offset + 1] = led_color.r;
            data_vec[led_offset + 2] = led_color.b;
        }

        self.send_feature_to_blinkstick(&data_vec[0..self.report_length])
    }

    /// Makes a specified led blink in a single color
    ///
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    /// * `delay` - The delay between turning the light on and off
    /// * `blinks` - The number of times the light will blink
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Panics
    /// The call to `blink_led_color` will panic if the specified `led` is out of bounds for the connected BlinkStick device.
    ///
    /// # Example
    /// Makes the 0th led blink 5 times, once every second, with a purple glow
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new().unwrap();
    /// blinkstick.blink_led_color(0, std::time::Duration::from_secs(1), 5, Color {r: 25, g: 0, b: 25}).unwrap();
    /// ```
    pub fn blink_led_color(&self, led: u8, delay: Duration, blinks: u32, color: Color) -> Result<(), FeatureError> {
        for _ in 0..blinks {
            self.set_led_color(led, color)?;
            std::thread::sleep(delay);
            self.set_led_color(led, Color { r: 0, g: 0, b: 0 })?;
            std::thread::sleep(delay);
        }

        Ok(())
    }

    /// Makes the specified leds blink in a single color
    ///
    /// # Arguments
    /// * `leds` - A vector of zero-indexed led numbers (within bounds for the BlinkStick product)
    /// * `delay` - The delay between turning the lights on and off
    /// * `blinks` - The number of times the lights will blink
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Panics
    /// The call to `blink_multiple_leds_color` will panic if any of the specified `leds` is out of bounds for the BlinkStick device.
    ///
    /// # Example
    /// Makes the zeroth and first led blink 2 times, once every 200 milliseconds, with a yellow glow
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new().unwrap();
    /// blinkstick.blink_multiple_leds_color(&vec![0, 1], std::time::Duration::from_millis(200), 2, Color {r: 50, g: 50, b: 0}).unwrap();
    /// ```
    pub fn blink_multiple_leds_color(
        &self,
        leds: &[u8],
        delay: Duration,
        blinks: u32,
        color: Color,
    ) -> Result<(), FeatureError> {
        for _ in 0..blinks {
            self.set_multiple_leds_color(leds, color)?;
            std::thread::sleep(delay);
            self.set_multiple_leds_color(leds, Color { r: 0, g: 0, b: 0 })?;
            std::thread::sleep(delay);
        }

        Ok(())
    }

    /// Makes all leds blink in a single color
    ///
    /// # Arguments
    /// * `delay` - The delay between turning the lights on and off
    /// * `blinks` - The number of times the lights will blink
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Example
    /// Makes all leds blink 2 times, once every 200 milliseconds, with a yellow glow
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new().unwrap();
    /// blinkstick.blink_all_leds_color(std::time::Duration::from_millis(200), 2, Color {r: 50, g: 50, b: 0}).unwrap();
    /// ```
    pub fn blink_all_leds_color(&self, delay: Duration, blinks: u32, color: Color) -> Result<(), FeatureError> {
        let leds: Vec<u8> = (0..self.max_leds).collect();
        self.blink_multiple_leds_color(&leds, delay, blinks, color)
    }

    /// Makes the specified led pulse from its current color to a specified color and back again
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color changes are interpolated between the old and new color value
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    ///
    /// # Panics
    /// The call to `pulse_led_color` will panic if the specified `led` is out of bounds for the connected BlinkStick device.
    /// The call to `pulse_led_color` will panic if the internal communication time is shorter then `duration`/`steps`.
    ///
    /// # Example
    /// Makes the 2nd led, pulse from an off state, to a blue glow, and then return back again to the off state with a two second animation time
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new().unwrap();
    /// blinkstick.pulse_led_color(2, std::time::Duration::from_secs(2), 20, Color {r: 0, g: 0, b: 155}).unwrap();
    /// ```
    pub fn pulse_led_color(&self, led: u8, duration: Duration, steps: u16, color: Color) -> Result<(), FeatureError> {
        let old_color = self.get_led_color(led)?;
        self.transform_led_color(led, duration / 2, steps, color)?;
        self.transform_led_color(led, duration / 2, steps, old_color)?;

        Ok(())
    }

    /// Makes the specified leds pulse to a single color and back to their original color
    ///
    /// # Arguments
    /// * `leds` - A vector of zero-indexed led numbers (within bounds for the BlinkStick product)
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color value is update during the transformation
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Panics
    /// The call to `pulse_multiple_leds_color` will panic if any of the specified `leds` is out of bounds for the BlinkStick device.
    /// The call to `pulse_multiple_leds_color` will panic if the internal communication time is shorter then `duration`/`steps`.
    ///
    /// # Example
    /// Gives the zeroth and fourth led a random color, and makes them pulse to a blue color
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new().unwrap();
    ///
    /// let mut colors: Vec<Color> = blinkstick.get_color_vec();
    /// colors[0] = BlinkStick::get_random_color();
    /// colors[4] = BlinkStick::get_random_color();
    ///
    /// blinkstick.set_all_leds_colors(&colors).unwrap();
    ///
    /// let color = Color {r: 0, g: 0, b: 55};
    /// blinkstick.pulse_multiple_leds_color(&vec![0, 4], std::time::Duration::from_secs(5), 50, color).unwrap();
    ///
    /// assert_eq!(blinkstick.get_led_color(0).unwrap(), colors[0]);
    /// assert_eq!(blinkstick.get_led_color(4).unwrap(), colors[4]);
    /// ```
    pub fn pulse_multiple_leds_color(
        &self,
        leds: &[u8],
        duration: Duration,
        steps: u16,
        color: Color,
    ) -> Result<(), FeatureError> {
        let old_colors = self.get_all_led_colors()?;

        self.transform_multiple_leds_color(leds, duration.div(2), steps, color)?;
        self.transform_all_leds_colors(duration.div(2), steps, &old_colors)
    }

    /// Makes all leds pulse between their current color and a specified color
    ///
    /// #Arguments
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color value is update during the transformation
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Panics
    /// The call to `pulse_all_leds_color` will panic if the internal communication time is shorter then `duration`/`steps`.
    ///
    /// # Example
    /// Makes every led pulse between being turned off and a green color
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new().unwrap();
    ///
    /// blinkstick.pulse_all_leds_color(std::time::Duration::from_secs(2), 25, Color {r: 0, g: 25, b: 0}).unwrap();
    ///
    /// assert_eq!(blinkstick.get_all_led_colors().unwrap(), vec![Color {r: 0, g: 0, b: 0}; blinkstick.max_leds as usize]);
    /// ```
    pub fn pulse_all_leds_color(
        &self,
        duration: Duration,
        steps: u16,
        target_color: Color,
    ) -> Result<(), FeatureError> {
        let old_colors = self.get_all_led_colors()?;

        self.transform_all_leds_color(duration.div(2), steps, target_color)?;
        self.transform_all_leds_colors(duration.div(2), steps, &old_colors)
    }

    /// Makes the specified led shift into a different color
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color value is update during the transformation
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Panics
    /// The call to `transform_led_color` will panic if the specified `led` is out of bounds for the connected BlinkStick device.
    /// The call to `transform_led_color` will panic if the internal communication time is shorter then `duration`/`steps`.
    ///
    /// Additionally, by choosing a very high `step` count, it makes the internal animation interval shorter then the function execution
    /// meaning that the animation would have taken longer then the specified duration to finish. Therefore, the function
    /// panics if this threshold is overstepped. A rule of thumb is for each second of animation, 100 steps is a softmax.
    ///
    /// # Example
    /// Makes the first led transform from a red color into a green color over a period of five seconds, with 50 color updates.
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new().unwrap();
    /// blinkstick.set_led_color(1, Color {r: 50, g: 0, b: 0}).unwrap();
    /// blinkstick.transform_led_color(1, std::time::Duration::from_secs(5), 50, Color {r: 0, g: 50, b: 0}).unwrap();
    /// ```
    pub fn transform_led_color(
        &self,
        led: u8,
        duration: Duration,
        steps: u16,
        target_color: Color,
    ) -> Result<(), FeatureError> {
        let interval = duration.div(steps as u32);
        let start_led_color = self.get_led_color(led)?;

        let gradient: Vec<Color> = calculate_gradients(start_led_color, target_color, steps);

        for color in gradient {
            let start = Instant::now();
            self.set_led_color(led, color)?;
            let elapsed = start.elapsed();

            let subtracted_duration = interval.saturating_sub(elapsed);
            if subtracted_duration != Duration::ZERO {
                std::thread::sleep(subtracted_duration);
            }
        }

        Ok(())
    }

    /// Transforms the color of all leds into a specified color on a per led basis
    ///
    /// # Arguments
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color value is update during the transformation
    /// * `colors` - A vector of `Color` with equal length to the number of leds available on the device.
    ///
    /// # Panics
    /// The call to `transform_all_leds_colors` will panic if the internal communication time is shorter then `duration`/`steps`.
    ///
    /// Additionally, by choosing a very high `step` count, it makes the internal animation interval shorter then the function execution
    /// meaning that the animation would have taken longer then the specified duration to finish. Therefore, the function
    /// panics if this threshold is overstepped. A rule of thumb is for each second of animation, 100 steps is a softmax.
    ///     
    /// # Example
    /// Sets a random color for each available led then transforms each individual led into a different random `Color`.
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new().unwrap();
    ///
    /// let mut colors: Vec<Color> = blinkstick.get_color_vec();
    /// for led in 0..blinkstick.max_leds as usize {
    ///     colors[led] = BlinkStick::get_random_color();
    /// }
    ///
    /// let mut new_colors: Vec<Color> = blinkstick.get_color_vec();
    /// for led in 0..blinkstick.max_leds as usize {
    ///     new_colors[led] = BlinkStick::get_random_color();
    /// }
    ///
    /// blinkstick.set_all_leds_colors(&colors).unwrap();
    /// blinkstick.transform_all_leds_colors(std::time::Duration::from_secs(2), 50, &new_colors).unwrap();
    /// ```
    pub fn transform_all_leds_colors(
        &self,
        duration: Duration,
        steps: u16,
        target_colors: &[Color],
    ) -> Result<(), FeatureError> {
        let mut led_gradients: Vec<Color> = Vec::with_capacity((self.max_leds as u16 * steps) as usize);
        for (led, target_color) in target_colors.iter().enumerate().take(self.max_leds as usize) {
            let current_led_color = self.get_led_color(led as u8)?;
            led_gradients.append(&mut calculate_gradients(current_led_color, *target_color, steps));
        }

        self.transform_leds(&led_gradients, duration, steps)
    }

    /// Transforms the color of all leds into a specified color
    ///
    /// # Arguments
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color value is update during the transformation
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Panics
    /// The call to `transform_all_leds_color` will panic if the internal communication time is shorter then `duration`/`steps`.
    ///
    /// # Example
    /// Transforms all leds from "off" to a blue `Color`.
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new().unwrap();
    ///
    /// blinkstick.transform_all_leds_color(std::time::Duration::from_secs(2), 50, Color { r: 0, g: 0, b: 100 }).unwrap();
    /// ```
    pub fn transform_all_leds_color(
        &self,
        duration: Duration,
        steps: u16,
        target_color: Color,
    ) -> Result<(), FeatureError> {
        let mut led_gradients: Vec<Color> = Vec::with_capacity((self.max_leds as u16 * steps) as usize);
        for led in 0..self.max_leds {
            let current_led_color = self.get_led_color(led as u8)?;
            led_gradients.append(&mut calculate_gradients(current_led_color, target_color, steps));
        }

        self.transform_leds(&led_gradients, duration, steps)
    }

    /// Performs the all leds transformation using a pre-computed gradient vector
    fn transform_leds(&self, led_gradients: &[Color], duration: Duration, steps: u16) -> Result<(), FeatureError> {
        let interval = duration.div(steps as u32);
        for step in 0..steps {
            let start = Instant::now();

            let test: Vec<Color> = led_gradients
                .iter()
                .skip(step as usize)
                .step_by(steps as usize)
                .copied()
                .collect();
            self.set_all_leds_colors(&test)?;

            let subtracted_duration = interval.saturating_sub(start.elapsed());
            if subtracted_duration != Duration::ZERO {
                std::thread::sleep(subtracted_duration);
            }
        }

        Ok(())
    }

    /// Transforms the color of the specified leds into a single color
    ///
    /// # Arguments
    /// * `leds` - A vector of zero-indexed led numbers (within bounds for the BlinkStick product)
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color value is update during the transformation
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Panics
    /// The call to `transform_multiple_leds_color` will panic if any of the specified `leds` is out of bounds for the BlinkStick device.
    /// The call to `transform_multiple_leds_color` will panic if the internal communication time is shorter then `duration`/`steps`.
    ///
    /// # Example
    /// Sets a random color for each available led then transforms it all into a single `Color`.
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new().unwrap();
    ///
    /// let mut colors: Vec<Color> = blinkstick.get_color_vec();
    ///
    /// for led in 0..blinkstick.max_leds as usize {
    ///     colors[led] = BlinkStick::get_random_color();
    /// }
    ///
    /// blinkstick.set_all_leds_colors(&colors).unwrap();
    ///
    /// let led_vec: Vec<u8> = (0..blinkstick.max_leds).collect();
    /// blinkstick.transform_multiple_leds_color(&led_vec, std::time::Duration::from_secs(2), 50, Color {r: 55, g: 0, b: 55}).unwrap();
    /// ```
    pub fn transform_multiple_leds_color(
        &self,
        leds: &[u8],
        duration: Duration,
        steps: u16,
        target_color: Color,
    ) -> Result<(), FeatureError> {
        let interval = duration.div(steps as u32);

        let mut led_gradients: Vec<Color> = Vec::with_capacity((leds.len() * steps as usize) as usize);
        for led in leds.iter() {
            let current_led_color = self.get_led_color(*led)?;
            led_gradients.append(&mut calculate_gradients(current_led_color, target_color, steps));
        }

        for step in 0..steps as usize {
            let start = Instant::now();
            let mut all_led_colors = self.get_all_led_colors()?;
            for (index, led) in leds.iter().enumerate() {
                all_led_colors[*led as usize] = led_gradients[index * steps as usize + step];
            }
            self.set_all_leds_colors(&all_led_colors)?;

            std::thread::sleep(interval.sub(start.elapsed()));
        }

        Ok(())
    }

    /// Makes the blinkstick device carousel. A Carousel utilizes all leds to transition between `start_color`, `stop_color` and back to `start_color`
    ///
    /// # Arguments
    /// * `start_color` - The start color to transition from
    /// * `stop_color` - The target color to transition to
    ///
    /// # Example
    ///
    /// Carousels the BlinkStick device Blue -> Green -> Blue 10 times
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::default();
    /// let color_one = Color { r: 0, g: 0, b: 50 };
    /// let color_two = Color {r: 0, g: 50, b: 0};
    /// for _ in 0..10 {
    ///     blinkstick.carousel(color_one, color_two, std::time::Duration::from_millis(20)).unwrap();
    /// }
    /// ```
    pub fn carousel(&self, start_color: Color, target_color: Color, delay: Duration) -> Result<(), FeatureError> {
        let mut carousel_colors = calculate_gradients(start_color, target_color, self.max_leds as u16);

        self.color_lap(&carousel_colors, &delay)?;
        carousel_colors.reverse();
        self.color_lap(&carousel_colors, &delay)
    }

    /// Helper function for carousel
    fn color_lap(&self, lap_colors: &[Color], delay: &Duration) -> Result<(), FeatureError> {
        self.set_led_color(0_u8, lap_colors[0])?;
        for i in 1..self.max_leds {
            std::thread::sleep(*delay);

            self.turn_off_led(i - 1)?;
            self.set_led_color(i, lap_colors[i as usize])?;
        }

        self.turn_off_led(self.max_leds - 1)
    }

    /// Gets the color of every single led on the BlinkStick device
    ///
    /// # Example
    /// Gets the color of every single led
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new().unwrap();
    ///    
    /// let random_color = BlinkStick::get_random_color();
    ///
    /// blinkstick.set_led_color(1, random_color).unwrap();
    /// blinkstick.set_led_color(2, random_color).unwrap();
    ///
    /// let led_colors = blinkstick.get_all_led_colors().unwrap();
    ///
    /// assert_ne!(led_colors[0], random_color);
    /// assert_eq!(led_colors[1], random_color);
    /// assert_eq!(led_colors[2], random_color);
    /// ```
    pub fn get_all_led_colors(&self) -> Result<Vec<Color>, FeatureError> {
        let buf = self.get_feature_from_blinkstick(0x6)?;

        let mut led_colors: Vec<Color> = Vec::with_capacity(self.max_leds as usize);
        for led in 0..self.max_leds as usize {
            let led_color_index = (led * 3) + 2;

            led_colors.push(Color {
                r: buf[led_color_index + 1],
                g: buf[led_color_index],
                b: buf[led_color_index + 2],
            });
        }
        Ok(led_colors)
    }

    /// Gets the color of a single led on the BlinkStick device
    ///
    /// # Example
    /// Gets the color of the zeroth led
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new().unwrap();
    ///    
    /// let random_color = BlinkStick::get_random_color();
    ///
    /// blinkstick.set_led_color(0, random_color).unwrap();
    ///
    /// let led_color = blinkstick.get_led_color(0).unwrap();
    ///
    /// assert_eq!(led_color, random_color);
    /// ```
    pub fn get_led_color(&self, led: u8) -> Result<Color, FeatureError> {
        let colors: Vec<Color> = self.get_all_led_colors()?;

        if led >= self.max_leds {
            panic!(
                "BlinkStick device does not contain led {}. Valid leds are 0-{} (zero-indexed)",
                led,
                self.max_leds - 1
            );
        }

        Ok(colors[led as usize])
    }

    fn send_feature_to_blinkstick(&self, feature: &[u8]) -> Result<(), FeatureError> {
        for _ in 0..5 {
            if self.device.send_feature_report(feature).is_ok() {
                return Ok(());
            }
        }

        // If we still dont have a successful attempt at communicating with the device
        // we try one last time after a short sleep
        std::thread::sleep(std::time::Duration::from_millis(10));
        if self.device.send_feature_report(feature).is_ok() {
            Ok(())
        } else {
            Err(FeatureError { kind: Send })
        }
    }

    fn get_feature_from_blinkstick(&self, id: u8) -> Result<[u8; REPORT_ARRAY_BYTES], FeatureError> {
        let mut buf = [0u8; REPORT_ARRAY_BYTES];
        buf[0] = id;

        for _ in 0..5 {
            if self.device.get_feature_report(&mut buf).is_ok() {
                return Ok(buf);
            }
        }

        // If we still dont have a successful attempt at communicating with the device
        // we try one last time after a short sleep
        std::thread::sleep(std::time::Duration::from_millis(10));
        if self.device.get_feature_report(&mut buf).is_ok() {
            Ok(buf)
        } else {
            Err(FeatureError { kind: Get })
        }
    }
}

fn calculate_gradients(start_color: Color, target_color: Color, steps: u16) -> Vec<Color> {
    (1..=steps)
        .into_iter()
        .map(|step| {
            let step_percent = step as f32 / steps as f32;
            Color {
                r: ((start_color.r as f32 * (1.0 - step_percent)) + (target_color.r as f32 * step_percent)) as u8,
                g: ((start_color.g as f32 * (1.0 - step_percent)) + (target_color.g as f32 * step_percent)) as u8,
                b: ((start_color.b as f32 * (1.0 - step_percent)) + (target_color.b as f32 * step_percent)) as u8,
            }
        })
        .collect()
}

#[cfg(test)]
mod blinkstick {
    use super::*;

    #[test]
    fn create_device_connection() {
        BlinkStick::new().expect("Could not create connection");
    }

    #[test]
    fn get_led_color() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        let color = Color { r: 17, g: 2, b: 3 };

        blinkstick.set_led_color(5, color).expect("Could not set led color");
        let led_color = blinkstick.get_led_color(5).expect("Could not get color from led");

        assert_eq!(led_color, color);
    }

    #[test]
    fn get_all_led_colors() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        let color = Color { r: 2, g: 2, b: 7 };

        blinkstick.set_all_leds_color(color).expect("Could not set led colors");
        let led_colors = blinkstick.get_all_led_colors().expect("Could not get led colors");

        assert_eq!(led_colors, vec![color; blinkstick.max_leds as usize]);
    }

    #[test]
    fn flash_multiple_leds_single_color() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        let led_vec = vec![0, 2, 4, 6];

        let color = Color { r: 10, g: 0, b: 0 };
        let default_color = Color { r: 0, g: 0, b: 0 };

        blinkstick
            .set_multiple_leds_color(&led_vec, color)
            .expect("Could not set led colors");
        let led_colors = blinkstick.get_all_led_colors().expect("Could not get led colors");

        let mut equality_vec = vec![default_color; blinkstick.max_leds as usize];
        for led in led_vec {
            equality_vec[led as usize] = color;
        }

        assert_eq!(led_colors, equality_vec);
    }

    #[test]
    #[should_panic]
    fn flash_multiple_leds_out_of_bounds() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        blinkstick
            .set_multiple_leds_color(&[blinkstick.max_leds], Color { r: 5, g: 5, b: 5 })
            .expect("Could not set led colors");
    }

    #[test]
    fn blink_led_color() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        let blink_led_color = Color { r: 25, g: 65, b: 100 };
        blinkstick
            .blink_led_color(3, std::time::Duration::from_millis(200), 5, blink_led_color)
            .expect("Could not blink led");

        let led_color = blinkstick.get_led_color(3).expect("Could not get led color");
        assert_eq!(led_color, Color { r: 0, g: 0, b: 0 });
    }

    #[test]
    fn blink_all_leds_color() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        let blink_led_color = Color { r: 25, g: 65, b: 100 };
        blinkstick
            .blink_all_leds_color(std::time::Duration::from_millis(200), 5, blink_led_color)
            .expect("Could not blink leds");
        assert_eq!(
            blinkstick.get_all_led_colors().expect("Could not get led colors"),
            vec![Color { r: 0, g: 0, b: 0 }; blinkstick.max_leds as usize]
        );
    }

    #[test]
    #[should_panic]
    fn blink_single_led_out_of_bounds() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        blinkstick
            .blink_led_color(
                blinkstick.max_leds,
                std::time::Duration::from_millis(200),
                5,
                Color { r: 10, g: 0, b: 10 },
            )
            .expect("Could not blink, as intended");
    }

    #[test]
    #[should_panic]
    fn blink_multiple_leds_out_of_bounds() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        blinkstick
            .blink_multiple_leds_color(
                &[blinkstick.max_leds],
                std::time::Duration::from_millis(200),
                5,
                Color { r: 5, g: 10, b: 10 },
            )
            .expect("Could not blink, as intended");
    }

    #[test]
    fn transform_led_color() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        let from_color = Color { r: 150, g: 150, b: 150 };
        let to_color = Color { r: 0, g: 0, b: 0 };

        blinkstick
            .set_led_color(2, from_color)
            .expect("Could not set led color");
        assert_eq!(
            blinkstick.get_led_color(2).expect("Could not get led color"),
            from_color
        );

        blinkstick
            .transform_led_color(2, Duration::from_secs(1), 25, to_color)
            .expect("Could not transform led");
        assert_eq!(blinkstick.get_led_color(2).expect("Could not get led color"), to_color);
    }

    #[test]
    fn transform_multiple_leds_color() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        let color_one = Color { r: 5, g: 5, b: 75 };
        let color_two = Color { r: 60, g: 111, b: 5 };

        let target_color = Color { r: 100, g: 0, b: 0 };

        blinkstick.set_led_color(3, color_one).expect("Could not set led color");
        blinkstick.set_led_color(5, color_two).expect("Could not set led color");

        assert_eq!(blinkstick.get_led_color(3).expect("Could not get led color"), color_one);
        assert_eq!(blinkstick.get_led_color(5).expect("Could not get led color"), color_two);

        blinkstick
            .transform_multiple_leds_color(&[3, 5], std::time::Duration::from_secs(4), 25, target_color)
            .expect("Could not transform leds");
        assert_eq!(
            blinkstick.get_led_color(3).expect("Could not get led color"),
            target_color
        );
        assert_eq!(
            blinkstick.get_led_color(5).expect("Could not get led color"),
            target_color
        );
    }

    #[test]
    fn pulse_led_color() {
        let blinkstick = BlinkStick::new().expect("Could not create connection");

        let from_color = Color { r: 50, g: 0, b: 0 };
        let to_color = Color { r: 0, g: 0, b: 155 };

        blinkstick
            .set_led_color(2, from_color)
            .expect("Could not set led color");
        assert_eq!(
            blinkstick.get_led_color(2).expect("Could not get led color"),
            from_color
        );
        blinkstick
            .pulse_led_color(2, Duration::from_secs(2), 25, to_color)
            .expect("Could not pulse led color");
        assert_eq!(
            blinkstick.get_led_color(2).expect("Could not get led color"),
            from_color
        );
    }
}
