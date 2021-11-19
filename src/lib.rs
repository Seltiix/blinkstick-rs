//! This crate provides a rust toolkit for interacting with the BlinkStick device.
//! The implementation should support all types of BlinkStick devices. It was however
//! implemented and tested using a BlinkStick Square. If a BlinkStick device acts incorrectly, please contact me.

#[allow(unused_imports)]
use std::{thread, time::Duration, time::Instant};

extern crate hidapi;

const VENDOR_ID: u16 = 0x20a0;
const PRODUCT_ID: u16 = 0x41e5;

#[derive(Debug, Clone, Copy, PartialEq)]
struct Color {
    r: u8,
    g: u8,
    b: u8
}

pub struct BlinkStick {
    device: hidapi::HidDevice,
    max_leds: u8
}

impl Drop for BlinkStick {
    fn drop(&mut self) {
        self.set_all_colors(0, 0, 0);
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
            Err(error) => panic!("Problem connecting to device: {:?}", error)
        };

        // Determines the number of leds for a device. The BlinkStick Flex has 32 leds with 3 channels, which is the maximum of any device.
        // 32 * 3 + 2 = 98 bytes
        let mut buf: [u8; 100] = [0; 100];
        buf[0] = 0x6;
        let bytes_read = device.get_feature_report(&mut buf).unwrap();

        // First two bytes are meta information
        let max_leds = ((bytes_read - 2) / 3) as u8;

        BlinkStick { 
            device, 
            max_leds
        }
    }

    /// Sets the RGB color of a single led
    /// 
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    /// * `r` - The red channel value (0-255)
    /// * `g` - The green channel value (0-255)
    /// * `b` - The blue channel value (0-255)
    /// # Panics
    /// The call to `set_color` will panic if the specifed `led` is out of bounds for the connected BlinkStick device.
    /// # Examples
    /// Sets the color of the 0th led to red
    /// ```no_run
    /// use blinkstick_rs_rs::BlinkStick;
    /// let blinkstick = BlinkStick::new();
    /// 
    /// blinkstick.set_color(0, 50, 0, 0);
    /// ```
    pub fn set_color(&self, led: u8, r: u8, g: u8, b: u8, ) {
        if led >= self.max_leds { panic!("Led {} is out of bounds for Blinkstick device", led) }

        self.device.send_feature_report(&[0x5, 0, led, r, g, b]).expect("Could not set the color of Blinkstick led");
    }  

    /// Sets the RGB color of one or more leds to a single color
    /// # Arguments
    /// * `leds` - A vector of zero-indexed led numbers (within bounds for the BlinkStick product)
    /// * `r` - The red channel value (0-255)
    /// * `g` - The green channel value (0-255)
    /// * `b` - The blue channel value (0-255)
    /// # Panics
    /// The call to set_unified_color will panic if any of the specified `leds` is out of bounds for the BlinkStick device.
    /// # Examples
    /// Sets the color of 0th, 2nd, 4th and 6th led to green.
    /// ```no_run
    /// use blinkstick_rs::BlinkStick;
    /// 
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.set_unified_color(&vec![0, 2, 4, 6], 0, 50, 0);
    /// ```
    pub fn set_unified_color(&self, leds: &Vec<u8>, r: u8, g: u8, b: u8) {
        let mut data_vec: [u8; 26] = [0; 26];
        data_vec[0] = 0x6;

        for led in leds {
            let led_offset: usize = ((led * 3) + 2).into();

            if led_offset >= ((self.max_leds * 3) + 2).into() {
                panic!("BlinkStick device does not contain led {}. Valid leds are 0-{} (zero-indexed)", led, self.max_leds - 1);
            }
            
            // Why whould G,R,B ever be a good idea?
            data_vec[led_offset + 0] = g;
            data_vec[led_offset + 1] = r;
            data_vec[led_offset + 2] = b;

        }

        self.device.send_feature_report(&data_vec).expect("Could not set the color of Blinkstick leds");
    }

    /// Sets the same color for all leds available on the BlinkStick device
    /// # Arguments
    /// * `r` - The red channel value (0-255)
    /// * `g` - The green channel value (0-255)
    /// * `b` - The blue channel value (0-255)
    /// # Examples
    /// Turns every led blue
    /// ```no_run
    /// use blinkstick_rs::BlinkStick;
    /// 
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.set_all_colors(0, 0, 50);
    /// ```
    pub fn set_all_colors(&self, r: u8, g: u8, b: u8) {
        self.set_unified_color(&(0..self.max_leds).collect(), r, g, b)
    }

    /// Makes a specified led blink in a single color
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    /// * `delay` - The delay between turning the light on and off
    /// * `blinks` - The number of times the light will blink
    /// * `r` - The red channel value (0-255)
    /// * `g` - The green channel value (0-255)
    /// * `b` - The blue channel value (0-255)
    /// # Panics
    /// The call to `blink_color` will panic if the specifed `led` is out of bounds for the connected BlinkStick device.
    /// # Examples
    /// Makes the 0th led blink 5 times, once every second, with a purple glow
    /// ```no_run
    /// use blinkstick_rs::BlinkStick;
    /// 
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.blink_color(0, std::time::Duration::from_secs(1), 5, 25, 0, 25);
    /// ```
    pub fn blink_color(&self, led: u8, delay: Duration, blinks: u32, r: u8, g: u8, b: u8) {
        for _ in 0..blinks {
            self.set_color(led, r, g, b);
            std::thread::sleep(delay);
            self.set_color(led, 0, 0, 0);
            std::thread::sleep(delay);
        }
    }

    /// Makes the specified leds blink in a single color
    /// # Arguments
    /// * `leds` - A vector of zero-indexed led numbers (within bounds for the BlinkStick product)
    /// * `delay` - The delay between turning the lights on and off
    /// * `blinks` - The number of times the lights will blink
    /// * `r` - The red channel value (0-255)
    /// * `g` - The green channel value (0-255)
    /// * `b` - The blue channel value (0-255)
    /// # Panics
    /// The call to `blink_unified_color` will panic if any of the specified `leds` is out of bounds for the BlinkStick device.
    /// # Examples
    /// Makes the 1st, 3rd, 5th led blink 2 times, once every 200 miliseconds, with a yellow glow
    /// ```no_run
    /// use blinkstick_rs::BlinkStick;
    /// 
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.blink_unified_color(&vec![1, 3, 5], std::time::Duration::from_millis(200), 2, 50, 50, 0);
    /// ```
    pub fn blink_unified_color(&self, leds: &Vec<u8>, delay: Duration, blinks: u32, r: u8, g: u8, b: u8) {
        for _ in 0..blinks {
            self.set_unified_color(&leds, r, g, b);
            std::thread::sleep(delay);
            self.set_unified_color(&leds, 0, 0, 0);
            std::thread::sleep(delay);
        }
    }

    /// Makes the specified led pulse from its current color to a specified color and back again
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color changes are interpolated between the old and new color value
    /// * `r` - The red channel value (0-255)
    /// * `g` - The green channel value (0-255)
    /// * `b` - The blue channel value (0-255)
    /// 
    /// 
    /// # Panics
    /// The call to `pulse_color` will panic if the specifed `led` is out of bounds for the connected BlinkStick device.
    /// 
    /// Additionally, by choosing a very high `step` count, it makes the internal animation interval shorter then the function execution
    /// meaning that the animation would have taken longer then the specified duration to finish. Therefore, the function
    /// panics if this threshold is overstepped. A rule of thumb is for each second of animation, 50 steps is a softmax.
    /// # Examples
    /// Makes the 2nd led, pulse from an off state, to a blue glow, and then return back again to the off state with a two second animation time
    /// ```no_run
    /// use blinkstick_rs::BlinkStick;
    /// 
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.pulse_color(2, std::time::Duration::from_secs(2), 20, 0, 0, 155);
    /// ```
    pub fn pulse_color(&self, led: u8, duration: Duration, steps: u64, r: u8, g: u8, b: u8) {
        let old_color = self.get_color(led);
        self.transform_color(led, duration/2, steps, r, g, b);
        self.transform_color(led, duration/2, steps, old_color.r, old_color.g, old_color.b);
    }

    /// Makes the specified led shift into a different color
    /// # Arguments
    /// * `led` - A zero-indexed led number (within bounds for the BlinkStick product)
    /// * `duration` - The time it takes for the entire animation cycle to finish
    /// * `steps` - The number of times the color value is update during the transformation
    /// * `r` - The red channel value (0-255)
    /// * `g` - The green channel value (0-255)
    /// * `b` - The blue channel value (0-255)
    /// 
    /// # Panics
    /// The call to `transform_color` will panic if the specifed `led` is out of bounds for the connected BlinkStick device.
    /// 
    /// Additionally, by choosing a very high `step` count, it makes the internal animation interval shorter then the function execution
    /// meaning that the animation would have taken longer then the specified duration to finish. Therefore, the function
    /// panics if this threshold is overstepped. A rule of thumb is for each second of animation, 100 steps is a softmax.
    /// 
    /// # Examples
    /// Makes the 1st led transform from a red color into a green color over a period of five seconds, with 50 color updates.
    /// ```no_run
    /// use blinkstick_rs::BlinkStick;
    /// 
    /// let blinkstick = BlinkStick::new();
    /// blinkstick.set_color(1, 50, 0, 0);
    /// blinkstick.transform_color(1, std::time::Duration::from_secs(5), 50, 0, 50, 0);
    /// ```
    pub fn transform_color(&self, led: u8, duration: Duration, steps: u64, r: u8, g: u8, b: u8) {
        
        let interval = duration.as_millis() as u64 / steps;
        for step in 0..steps {
            let start = Instant::now();
            let led_color = self.get_color(led);

            let new_r: u8 = self.move_color(led_color.r, r, step, steps);
            let new_g: u8 = self.move_color(led_color.g, g, step, steps);
            let new_b: u8 = self.move_color(led_color.b, b, step, steps);

            self.set_color(led, new_r, new_g, new_b);
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
    fn get_colors(&self) -> Vec<Color> {

        let mut buf: [u8; 100] = [0; 100];
        buf[0] = 0x6;
        self.device.get_feature_report(&mut buf).unwrap();


        let mut led_colors: Vec<Color> = Vec::with_capacity(self.max_leds as usize);
        for led in 0..self.max_leds as usize {
            let led_color_index = (led * 3) + 2;

            led_colors.push(Color { r: buf[led_color_index + 1], g: buf[led_color_index], b: buf[led_color_index + 2] });
        }
        led_colors
    }

    fn get_color(&self, led: u8) -> Color {
        let colors: Vec<Color> = self.get_colors();

        if led >= self.max_leds {
            panic!("BlinkStick device does not contain led {}. Valid leds are 0-{} (zero-indexed)", led, self.max_leds - 1);
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
    fn get_color() {
        let blinkstick = BlinkStick::new();

        blinkstick.set_color(5, 17, 2, 3);
        let led_color = blinkstick.get_color(5);
        assert_eq!(led_color, Color {r: 17, g: 2, b: 3});

        blinkstick.set_color(5, 0, 0, 0);
    }

    #[test]
    #[serial]
    fn get_colors() {
        let blinkstick = BlinkStick::new();

        blinkstick.set_all_colors(2, 2, 7);
        let led_colors = blinkstick.get_colors();

        assert_eq!(led_colors, vec![Color {r: 2, g: 2, b: 7}; blinkstick.max_leds as usize]);

        blinkstick.set_all_colors(0, 0, 0);
        
    }

    #[test]
    #[serial]
    fn flash_single_led() {
        let blinkstick = BlinkStick::new();

        blinkstick.set_color(0, 15, 0, 2);
        let led_color = blinkstick.get_color(0);
        assert_eq!(led_color, Color {r: 15, g: 0, b: 2});

        blinkstick.set_color(0, 0, 0, 0);
    }

    #[test]
    #[serial]
    fn flash_all_leds() {
        let blinkstick = BlinkStick::new();

        blinkstick.set_all_colors(10, 0, 10);
        let led_colors = blinkstick.get_colors();

        assert_eq!(led_colors, vec![Color {r: 10, g: 0, b:10}; blinkstick.max_leds as usize]);

        blinkstick.set_all_colors(0, 0, 0);
    }

    #[test]
    #[serial]
    fn flash_multiple_leds_single_color() {
        let blinkstick = BlinkStick::new();

        let led_vec = vec![0, 2, 4, 6];

        blinkstick.set_unified_color(&led_vec, 10, 0, 0);
        let led_colors = blinkstick.get_colors();

        let mut equality_vec = vec![Color {r: 0, g: 0, b: 0}; blinkstick.max_leds as usize];
        for led in led_vec {
            equality_vec[led as usize] = Color {r: 10, g: 0, b: 0};
        }
        
        assert_eq!(led_colors, equality_vec);

        blinkstick.set_unified_color(&vec![0,2,4,6], 0, 0, 0);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn flash_multiple_leds_out_of_bounds() {
        let blinkstick = BlinkStick::new();

        blinkstick.set_unified_color(&vec![blinkstick.max_leds], 10, 0, 10);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn blink_single_led_out_of_bounds() {
        let blinkstick = BlinkStick::new();

        blinkstick.blink_color(blinkstick.max_leds, std::time::Duration::from_millis(200), 5, 10, 0, 10);
    }

    #[test]
    #[serial]
    #[should_panic]
    fn blink_multiple_leds_out_of_bounds() {
        let blinkstick = BlinkStick::new();

        blinkstick.blink_unified_color(&vec![blinkstick.max_leds], std::time::Duration::from_millis(200), 5, 10, 0, 10)
    }

    #[test]
    #[serial]
    fn transform_color() {
        let blinkstick = BlinkStick::new();

        blinkstick.set_color(2, 150, 150, 150);
        assert_eq!(blinkstick.get_color(2), Color {r: 150, g: 150, b: 150});

        blinkstick.transform_color(2, Duration::from_secs(1), 100, 0, 0, 0);
        assert_eq!(blinkstick.get_color(2), Color {r: 0, g: 0, b: 0});
    }

    #[test]
    #[serial]
    fn pulse_color() {
        let blinkstick = BlinkStick::new();

        blinkstick.set_color(2, 50, 0, 0);
        assert_eq!(blinkstick.get_color(2), Color {r: 50, g: 0, b: 0});
        blinkstick.pulse_color(2, Duration::from_secs(2), 100, 0, 0, 155);
        assert_eq!(blinkstick.get_color(2), Color {r: 50, g: 0, b: 0});
    }

}
