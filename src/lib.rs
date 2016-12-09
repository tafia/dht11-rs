extern crate cupi;

mod errors;

use std::thread;
use std::time::{Duration, SystemTime};
use cupi::{CuPi, delay_ms, DigitalRead, DigitalWrite, Logic, PinOptions, PinInput};
use errors::{Error, Result};

pub struct DHT11 {
    pin: PinOptions,
}

impl DHT11 {

    /// Initialize a new connection on pin with DHT11 sensor
    pub fn new(pin_num: usize) -> Result<DHT11> {
        Ok(DHT11 {
            pin: CuPi::new()?.pin(pin_num)?,
        })
    }

    /// Reads humidity and temperature
    pub fn read(&mut self) -> Result<Response> {

        // send init signal
        {
            let mut output = self.pin.output();
            output.digital_write(Logic::Low)?;
            delay_ms(18);
            output.digital_write(Logic::High)?;
            delay_usec(40);
        }

        // get sensor response
        let mut intervals = [0; 40];
        {
            let mut input = self.pin.input();
            next_pulse(&mut input, Logic::Low)?;
            next_cycle_ns(&mut input, 80, 80)?; // init response
            for t in intervals.iter_mut() {
                *t = next_cycle_ns(&mut input, 50, 26)?; // bit data
            }
        }

        // convert intervals to bytes
        let mut bytes = [0u8; 5];
        for (val, ch) in bytes.iter_mut().zip(intervals.chunks(8)) {
            for t in ch {
                *val <<= 1;
                if *t > 100_000 { *val |= 1; }
            }
        }

        Ok(Response { bytes: bytes })
    }

}

pub struct Response {
    bytes: [u8; 5]
}

impl Response {
    pub fn is_valid(&self) -> bool {
        let sum: u16 = self.bytes.iter().take(4).map(|b| *b as u16).sum();
        self.bytes[4] as u16 == sum & 0x00FF
    }
    pub fn get_temperature(&self) -> f32 {
        self.bytes[2] as f32 + self.bytes[3] as f32 / 1000.
    }
    pub fn get_humidity(&self) -> f32 {
        self.bytes[0] as f32 + self.bytes[1] as f32 / 1000.
    }
}

fn delay_usec(usec: u32) {
    thread::sleep(Duration::new(0, usec * 1000));
}
                
fn next_pulse(input: &mut PinInput, level: Logic) -> Result<()> {
    for _ in 0..500 {
        match (level, input.digital_read()?) {
            (Logic::Low, Logic::Low) | (Logic::High, Logic::High) => return Ok(()),
            _ => delay_usec(1),
        }
    }
    Err(Error::TimeOut)
}

/// Measures the duration in nanoseconds of a cycle with indicative low and high durations
fn next_cycle_ns(input: &mut PinInput, low_us: u32, high_us: u32) -> Result<u32> {
    let start = SystemTime::now();
    delay_usec(low_us * 3 / 4);
    next_pulse(input, Logic::High)?; 
    delay_usec(high_us * 3 / 4);
    next_pulse(input, Logic::Low)?; 
    Ok(start.elapsed().unwrap().subsec_nanos())
}
