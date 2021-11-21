//! This crate provides a rust toolkit for interacting with the BlinkStick device.
//! The implementation should support all types of BlinkStick devices. It was however
//! implemented and tested using a BlinkStick Square. If a BlinkStick device acts incorrectly, please contact me.

use rand::Rng;
#[allow(unused_imports)]
use std::{thread, time::Duration, time::Instant};

extern crate hidapi;

const VENDOR_ID: u16 = 0x20a0;
const PRODUCT_ID: u16 = 0x41e5;

const REPORT_ARRAY_BYTES: usize = 100;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub struct BlinkStick {
    device: hidapi::HidDevice,
    pub max_leds: u8,
    report_length: usize
}

impl Drop for BlinkStick {
    fn drop(&mut self) {
        self.set_all_leds_color(Color { r: 0, g: 0, b: 0 });
    }
}

impl BlinkStick {
    /// Opens communication with a `BlinkStick Device`
    /// # Panics
    /// When there is no connected BlinkStick device, the call to new will panic.
    pub fn new() -> BlinkStick {
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

        BlinkStick { device, max_leds,  report_length }
    }

    /// Generates a random color
    ///
    /// # Example
    /// Returns a random `Color`
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new();
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
    /// let blinkstick = BlinkStick::new();
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
    /// The call to `set_led_color` will panic if the specifed `led` is out of bounds for the connected BlinkStick device.
    ///
    /// # Example
    /// Sets the color of the 0th led to red
    /// ```no_run
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new();
    ///
    /// blinkstick.set_led_color(0, Color {r: 50, g: 0, b: 0});
    /// ```
    pub fn set_led_color(&self, led: u8, color: Color) {
        if led >= self.max_leds {
            panic!("Led {} is out of bounds for Blinkstick device", led)
        }

        self.device
            .send_feature_report(&[0x5, 0, led, color.r, color.g, color.b])
            .expect("Could not set the color of Blinkstick led");
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
    /// ```no_run
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.set_multiple_leds_color(&vec![0, 2, 4, 6], Color {r: 0, g: 50, b: 0});
    /// ```
    pub fn set_multiple_leds_color(&self, leds: &Vec<u8>, color: Color) {
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

            // Why whould G,R,B ever be a good idea?
            data_vec[led_offset + 0] = color.g;
            data_vec[led_offset + 1] = color.r;
            data_vec[led_offset + 2] = color.b;
        }

        self.device
            .send_feature_report(&data_vec[0..self.report_length])
            .expect("Could not set the color of Blinkstick leds");
    }

    /// Sets the same color for all leds available on the BlinkStick device
    ///
    /// # Arguments
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Example
    /// Turns every led blue
    /// ```no_run
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.set_all_leds_color(Color {r: 0, g: 0, b: 50});
    /// ```
    pub fn set_all_leds_color(&self, color: Color) {
        self.set_multiple_leds_color(&(0..self.max_leds).collect(), color)
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
    /// let blinkstick = BlinkStick::new();
    ///
    /// let mut colors: Vec<Color> = blinkstick.get_color_vec();
    ///
    /// for led in 0..blinkstick.max_leds as usize {
    ///    colors[led] = BlinkStick::get_random_color();
    /// }
    ///
    /// blinkstick.set_all_leds_colors(&colors);
    /// ```
    pub fn set_all_leds_colors(&self, colors: &Vec<Color>) {
        let mut data_vec: [u8; REPORT_ARRAY_BYTES] = [0; REPORT_ARRAY_BYTES];
        data_vec[0] = 0x6;

        for led in 0..self.max_leds as usize {
            let led_offset = (led * 3) + 2;

            if led_offset >= ((self.max_leds * 3) + 2).into() {
                panic!(
                    "BlinkStick device does not contain led {}. Valid leds are 0-{} (zero-indexed)",
                    led,
                    self.max_leds - 1
                );
            }

            // Why whould G,R,B ever be a good idea?
            data_vec[led_offset + 0] = colors[led].g;
            data_vec[led_offset + 1] = colors[led].r;
            data_vec[led_offset + 2] = colors[led].b;
        }

        self.device
            .send_feature_report(&data_vec[0..self.report_length])
            .expect("Could not set the color of Blinkstick leds");
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
    /// The call to `blink_led_color` will panic if the specifed `led` is out of bounds for the connected BlinkStick device.
    ///
    /// # Example
    /// Makes the 0th led blink 5 times, once every second, with a purple glow
    /// ```no_run
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.blink_led_color(0, std::time::Duration::from_secs(1), 5, Color {r: 25, g: 0, b: 25});
    /// ```
    pub fn blink_led_color(&self, led: u8, delay: Duration, blinks: u32, color: Color) {
        for _ in 0..blinks {
            self.set_led_color(led, color);
            std::thread::sleep(delay);
            self.set_led_color(led, Color { r: 0, g: 0, b: 0 });
            std::thread::sleep(delay);
        }
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
    /// Makes the 1st, 3rd, 5th led blink 2 times, once every 200 miliseconds, with a yellow glow
    /// ```no_run
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.blink_multiple_leds_color(&vec![1, 3, 5], std::time::Duration::from_millis(200), 2, Color {r: 50, g: 50, b: 0});
    /// ```
    pub fn blink_multiple_leds_color(&self, leds: &Vec<u8>, delay: Duration, blinks: u32, color: Color) {
        for _ in 0..blinks {
            self.set_multiple_leds_color(&leds, color);
            std::thread::sleep(delay);
            self.set_multiple_leds_color(&leds, Color { r: 0, g: 0, b: 0 });
            std::thread::sleep(delay);
        }
    }

    /// Makes all leds blink in a single color
    ///
    /// # Arguments
    /// * `delay` - The delay between turning the lights on and off
    /// * `blinks` - The number of times the lights will blink
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Example
    /// Makes all leds blink 2 times, once every 200 miliseconds, with a yellow glow
    /// ```no_run
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.blink_all_leds_color(std::time::Duration::from_millis(200), 2, Color {r: 50, g: 50, b: 0});
    /// ```
    pub fn blink_all_leds_color(&self, delay: Duration, blinks: u32, color: Color) {
        self.blink_multiple_leds_color(&(0..self.max_leds).collect(), delay, blinks, color)
    }

    // @TODO
    //pub fn pulse_multiple_leds_color()
    //pub fn pulse_all_colors()
    //pub fn transform_all_leds_colors()

    /// Makes the specified led pulse from its current color to a specified color and back again
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color changes are interpolated between the old and new color value
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    ///
    /// # Panics
    /// The call to `pulse_led_color` will panic if the specifed `led` is out of bounds for the connected BlinkStick device.
    ///
    /// Additionally, by choosing a very high `step` count, it makes the internal animation interval shorter then the function execution
    /// meaning that the animation would have taken longer then the specified duration to finish. Therefore, the function
    /// panics if this threshold is overstepped. A rule of thumb is for each second of animation, 50 steps is a softmax.
    ///
    /// # Example
    /// Makes the 2nd led, pulse from an off state, to a blue glow, and then return back again to the off state with a two second animation time
    /// ```no_run
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.pulse_led_color(2, std::time::Duration::from_secs(2), 20, Color {r: 0, g: 0, b: 155});
    /// ```
    pub fn pulse_led_color(&self, led: u8, duration: Duration, steps: u64, color: Color) {
        let old_color = self.get_led_color(led);
        self.transform_led_color(led, duration / 2, steps, color);
        self.transform_led_color(led, duration / 2, steps, old_color);
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
    /// The call to `blink_multiple_leds_color` will panic if any of the specified `leds` is out of bounds for the BlinkStick device.
    ///
    /// Additionally, by choosing a very high `step` count, it makes the internal animation interval shorter then the function execution
    /// meaning that the animation would have taken longer then the specified duration to finish. Therefore, the function
    /// panics if this threshold is overstepped. A rule of thumb is for each second of animation, 100 steps is a softmax.
    /// 
    /// # Example
    /// Gives every led a random color, and make it pulse to a blue color
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color}; 
    ///
    /// let blinkstick = BlinkStick::new();
    /// 
    /// let mut colors: Vec<Color> = blinkstick.get_color_vec();
    /// for led in 0..blinkstick.max_leds as usize {
    ///     colors[led] = BlinkStick::get_random_color();
    /// }
    /// blinkstick.set_all_leds_colors(&colors);
    /// 
    /// let led_vec = (0..blinkstick.max_leds).collect();
    /// let color = Color {r: 0, g: 0, b: 55};
    /// blinkstick.pulse_multiple_leds_color(&led_vec, std::time::Duration::from_secs(5), 50, color);
    /// ```
    pub fn pulse_multiple_leds_color(&self, leds: &Vec<u8>, duration: Duration, steps: u64, color: Color) {
        let old_colors = self.get_all_led_colors();

        self.transform_multiple_leds_color(leds, duration / 2, steps, color);
        self.transform_all_leds_colors(duration / 2, steps, &old_colors)
    }

    /// Makes the specified led shift into a different color
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color value is update during the transformation
    /// * `color` - A struct holding color values for R,G and B channel respectively
    ///
    /// # Panics
    /// The call to `transform_led_color` will panic if the specifed `led` is out of bounds for the connected BlinkStick device.
    ///
    /// Additionally, by choosing a very high `step` count, it makes the internal animation interval shorter then the function execution
    /// meaning that the animation would have taken longer then the specified duration to finish. Therefore, the function
    /// panics if this threshold is overstepped. A rule of thumb is for each second of animation, 100 steps is a softmax.
    ///
    /// # Example
    /// Makes the 1st led transform from a red color into a green color over a period of five seconds, with 50 color updates.
    /// ```no_run
    /// use blinkstick_rs::{BlinkStick, Color};
    ///
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.set_led_color(1, Color {r: 50, g: 0, b: 0});
    /// blinkstick.transform_led_color(1, std::time::Duration::from_secs(5), 50, Color {r: 0, g: 50, b: 0});
    /// ```
    pub fn transform_led_color(&self, led: u8, duration: Duration, steps: u64, color: Color) {
        let interval = duration.as_millis() as u64 / steps;
        for step in 0..steps {
            let start = Instant::now();
            let led_color = self.get_led_color(led);

            let new_color = Color {
                r: self.move_color(led_color.r, color.r, step, steps),
                g: self.move_color(led_color.g, color.g, step, steps),
                b: self.move_color(led_color.b, color.b, step, steps),
            };

            self.set_led_color(led, new_color);
            let elapsed = start.elapsed().as_millis() as u64;

            if elapsed > interval {
                panic!("Executing a single color move took {} miliseconds, whilst the interval is set to {} miliseconds. Please reduce the number steps or increase the animation time.", elapsed, interval)
            }

            std::thread::sleep(Duration::from_millis(interval - elapsed));
        }
    }

    /// Transforms the color of all leds into a specified color
    /// 
    /// # Arguments
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color value is update during the transformation
    /// * `colors` - A vector of `Color` with equal length to the number of leds available on the device.
    /// 
    /// # Panics
    /// The call to `blink_multiple_leds_color` will panic if any of the specified `leds` is out of bounds for the BlinkStick device.
    ///
    /// Additionally, by choosing a very high `step` count, it makes the internal animation interval shorter then the function execution
    /// meaning that the animation would have taken longer then the specified duration to finish. Therefore, the function
    /// panics if this threshold is overstepped. A rule of thumb is for each second of animation, 100 steps is a softmax.
    ///     
    //// # Example
    /// Sets a random color for each available led then transforms each individual led into a different random `Color`.
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new();
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
    /// blinkstick.set_all_leds_colors(&colors);
    /// blinkstick.transform_all_leds_colors(std::time::Duration::from_secs(2), 200, &new_colors);
    /// ```
    pub fn transform_all_leds_colors(&self, duration: Duration, steps: u64, colors: &Vec<Color>) {
        let interval = duration.as_millis() as u64 / steps;

        for step in 0..steps {
            let start = Instant::now();

            let mut led_colors = self.get_all_led_colors();
            for led in 0..self.max_leds as usize {
                let led_color = &mut led_colors[led as usize];

                led_color.r = self.move_color(led_color.r, colors[led].r, step, steps);
                led_color.g = self.move_color(led_color.g, colors[led].g, step, steps);
                led_color.b = self.move_color(led_color.b, colors[led].b, step, steps);
            }

            self.set_all_leds_colors(&led_colors);

            let elapsed = start.elapsed().as_millis() as u64;

            if elapsed > interval {
                panic!("Executing a single color move took {} miliseconds, whilst the interval is set to {} miliseconds. Please reduce the number steps or increase the animation time.", elapsed, interval)
            }

            std::thread::sleep(Duration::from_millis(interval - elapsed));
        }
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
    /// The call to `blink_multiple_leds_color` will panic if any of the specified `leds` is out of bounds for the BlinkStick device.
    ///
    /// Additionally, by choosing a very high `step` count, it makes the internal animation interval shorter then the function execution
    /// meaning that the animation would have taken longer then the specified duration to finish. Therefore, the function
    /// panics if this threshold is overstepped. A rule of thumb is for each second of animation, 100 steps is a softmax.
    ///
    /// # Example
    /// Sets a random color for each available led then transforms it all into a single `Color`.
    /// ```
    /// use blinkstick_rs::{BlinkStick, Color};
    /// let blinkstick = BlinkStick::new();
    ///
    /// let mut colors: Vec<Color> = blinkstick.get_color_vec();
    ///
    /// for led in 0..blinkstick.max_leds as usize {
    ///     colors[led] = BlinkStick::get_random_color();
    /// }
    ///
    /// blinkstick.set_all_leds_colors(&colors);
    ///
    /// let led_vec = (0..blinkstick.max_leds).collect();
    /// blinkstick.transform_multiple_leds_color(&led_vec, std::time::Duration::from_secs(2), 200, Color {r: 55, g: 0, b: 55});
    /// ```
    pub fn transform_multiple_leds_color(
        &self,
        leds: &Vec<u8>,
        duration: Duration,
        steps: u64,
        color: Color,
    ) {
        let interval = duration.as_millis() as u64 / steps;

        for step in 0..steps {
            let start = Instant::now();

            let mut led_colors = self.get_all_led_colors();
            for led in leds {
                let led_color = &mut led_colors[*led as usize];

                led_color.r = self.move_color(led_color.r, color.r, step, steps);
                led_color.g = self.move_color(led_color.g, color.g, step, steps);
                led_color.b = self.move_color(led_color.b, color.b, step, steps);
            }

            self.set_all_leds_colors(&led_colors);

            let elapsed = start.elapsed().as_millis() as u64;

            if elapsed > interval {
                panic!("Executing a single color move took {} miliseconds, whilst the interval is set to {} miliseconds. Please reduce the number steps or increase the animation time.", elapsed, interval)
            }

            std::thread::sleep(Duration::from_millis(interval - elapsed));
        }
    }

    // Shifts a color value a single step in the direction of the targe, depending on the amount of remaining steps
    fn move_color(&self, color_value: u8, target_value: u8, step: u64, steps: u64) -> u8 {
        let diff;
        let ascending = color_value <= target_value;

        if ascending {
            diff = target_value - color_value;
        } else {
            diff = color_value - target_value;
        }

        let step_size = diff as u64 / (steps - step);

        if ascending {
            color_value + step_size as u8
        } else {
            color_value - step_size as u8
        }
    }

    // Gets the
    pub fn get_all_led_colors(&self) -> Vec<Color> {
        let mut buf: [u8; REPORT_ARRAY_BYTES] = [0; REPORT_ARRAY_BYTES];
        buf[0] = 0x6;
        self.device.get_feature_report(&mut buf).unwrap();

        let mut led_colors: Vec<Color> = Vec::with_capacity(self.max_leds as usize);
        for led in 0..self.max_leds as usize {
            let led_color_index = (led * 3) + 2;

            led_colors.push(Color {
                r: buf[led_color_index + 1],
                g: buf[led_color_index],
                b: buf[led_color_index + 2],
            });
        }
        led_colors
    }

    pub fn get_led_color(&self, led: u8) -> Color {
        let colors: Vec<Color> = self.get_all_led_colors();

        if led >= self.max_leds {
            panic!(
                "BlinkStick device does not contain led {}. Valid leds are 0-{} (zero-indexed)",
                led,
                self.max_leds - 1
            );
        }

        colors[led as usize]
    }
}

#[cfg(test)]
mod blinkstick {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn create_device_connection() {
        BlinkStick::new();
    }

    #[test]
    #[serial]
    fn get_led_color() {
        let blinkstick = BlinkStick::new();

        let color = Color { r: 17, g: 2, b: 3 };

        blinkstick.set_led_color(5, color);
        let led_color = blinkstick.get_led_color(5);

        assert_eq!(led_color, color);
    }

    #[test]
    #[serial]
    fn get_all_led_colors() {
        let blinkstick = BlinkStick::new();

        let color = Color { r: 2, g: 2, b: 7 };

        blinkstick.set_all_leds_color(color);
        let led_colors = blinkstick.get_all_led_colors();

        assert_eq!(led_colors, vec![color; blinkstick.max_leds as usize]);
    }

    #[test]
    #[serial]
    fn flash_multiple_leds_single_color() {
        let blinkstick = BlinkStick::new();

        let led_vec = vec![0, 2, 4, 6];

        let color = Color { r: 10, g: 0, b: 0 };
        let default_color = Color { r: 0, g: 0, b: 0 };

        blinkstick.set_multiple_leds_color(&led_vec, color);
        let led_colors = blinkstick.get_all_led_colors();

        let mut equality_vec = vec![default_color; blinkstick.max_leds as usize];
        for led in led_vec {
            equality_vec[led as usize] = color;
        }

        assert_eq!(led_colors, equality_vec);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn flash_multiple_leds_out_of_bounds() {
        let blinkstick = BlinkStick::new();

        blinkstick.set_multiple_leds_color(&vec![blinkstick.max_leds], Color { r: 5, g: 5, b: 5 });
    }

    #[test]
    #[serial]
    fn blink_led_color() {
        let blinkstick = BlinkStick::new();

        let blink_led_color = Color {
            r: 25,
            g: 65,
            b: 100,
        };
        blinkstick.blink_led_color(3, std::time::Duration::from_millis(200), 5, blink_led_color);

        assert_eq!(blinkstick.get_led_color(3), Color {r: 0, g: 0, b: 0});
    }

    #[test]
    #[serial]
    fn blink_all_leds_color() {
        let blinkstick = BlinkStick::new();

        let blink_led_color = Color {
            r: 25,
            g: 65,
            b: 100,
        };
        blinkstick.blink_all_leds_color(std::time::Duration::from_millis(200), 5, blink_led_color);

        assert_eq!(blinkstick.get_all_led_colors(), vec![Color {r: 0, g: 0, b: 0}; blinkstick.max_leds as usize]);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn blink_single_led_out_of_bounds() {
        let blinkstick = BlinkStick::new();

        blinkstick.blink_led_color(
            blinkstick.max_leds,
            std::time::Duration::from_millis(200),
            5,
            Color { r: 10, g: 0, b: 10 },
        );
    }

    #[test]
    #[serial]
    #[should_panic]
    fn blink_multiple_leds_out_of_bounds() {
        let blinkstick = BlinkStick::new();

        blinkstick.blink_multiple_leds_color(
            &vec![blinkstick.max_leds],
            std::time::Duration::from_millis(200),
            5,
            Color { r: 5, g: 10, b: 10 },
        );
    }

    #[test]
    #[serial]
    fn transform_led_color() {
        let blinkstick = BlinkStick::new();

        let from_color = Color {
            r: 150,
            g: 150,
            b: 150,
        };
        let to_color = Color { r: 0, g: 0, b: 0 };

        blinkstick.set_led_color(2, from_color);
        assert_eq!(blinkstick.get_led_color(2), from_color);

        blinkstick.transform_led_color(2, Duration::from_secs(1), 25, to_color);
        assert_eq!(blinkstick.get_led_color(2), to_color);
    }

    #[test]
    #[serial]
    fn transform_multiple_leds_color() {
        let blinkstick = BlinkStick::new();

        let color_one = Color { r: 5, g: 5, b: 75 };
        let color_two = Color {
            r: 60,
            g: 111,
            b: 5,
        };

        let target_color = Color { r: 100, g: 0, b: 0 };

        blinkstick.set_led_color(3, color_one);
        blinkstick.set_led_color(5, color_two);

        assert_eq!(blinkstick.get_led_color(3), color_one);
        assert_eq!(blinkstick.get_led_color(5), color_two);

        blinkstick.transform_multiple_leds_color(
            &vec![3, 5],
            std::time::Duration::from_secs(4),
            25,
            target_color,
        );

        assert_eq!(blinkstick.get_led_color(3), target_color);
        assert_eq!(blinkstick.get_led_color(5), target_color);
    }

    #[test]
    #[serial]
    fn pulse_led_color() {
        let blinkstick = BlinkStick::new();

        let from_color = Color { r: 50, g: 0, b: 0 };
        let to_color = Color { r: 0, g: 0, b: 155 };

        blinkstick.set_led_color(2, from_color);
        assert_eq!(blinkstick.get_led_color(2), from_color);
        blinkstick.pulse_led_color(2, Duration::from_secs(2), 25, to_color);
        assert_eq!(blinkstick.get_led_color(2), from_color);
    }
}
