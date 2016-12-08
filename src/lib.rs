extern crate sysfs_gpio;
#[macro_use]
extern crate error_chain;

mod errors;

use sysfs_gpio::{Pin, Direction};
use sysfs_gpio::Error::Unexpected;
use std::thread;
use std::time::{Duration, SystemTime};

pub struct DHT11 {
    pin: Pin,
}

impl DHT11 {

    /// Initialize a new connection on pin with DHT11 sensor
    pub fn new(pin_num: u64) -> DHT11 {
        DHT11 {
            pin: Pin::new(pin_num),
        }
    }

    /// Reads humidity and temperature
    pub fn read(&mut self) -> errors::Result<Response> {

        let mut intervals = [0; 40];
        if let Err(e) = self.pin.with_exported(|| {

            // send init signal
            self.pin.set_direction(Direction::Out)?;
            self.pin.set_value(0)?;
            thread::sleep(Duration::new(0, 18_000_000));
            self.pin.set_value(1)?;
            thread::sleep(Duration::new(0, 40_000));

            // getting sensor response
            self.pin.set_direction(Direction::In)?;
            self.next_pulse(0)?;
            self.next_cycle_ns(80_000, 80_000)?; // init response

            // 40 bit data
            for t in intervals.iter_mut() {
                *t = self.next_cycle_ns(50_000, 26_000)?; // bit data
            }
            Ok(())

        }) {
            println!("intervals: {:?}", intervals.as_ref());
            return Err(e.into());
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

    fn next_pulse(&self, level: u8) -> ::sysfs_gpio::Result<()> {
        for _ in 0..200 {
            if self.pin.get_value()? == level { return Ok(()); }
            thread::sleep(Duration::new(0, 1000));
        }
        Err(Unexpected("Timeout".to_string()))
    }

    /// Measures the duration in nanoseconds of a cycle with indicative low and high durations
    fn next_cycle_ns(&self, low_ns: u32, high_ns: u32) -> ::sysfs_gpio::Result<u32> {
        let start = SystemTime::now();
        thread::sleep(Duration::new(0, low_ns / 2));
        self.next_pulse(1)?;
        thread::sleep(Duration::new(0, high_ns / 2));
        self.next_pulse(0)?;
        Ok(start.elapsed().unwrap().subsec_nanos())
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
