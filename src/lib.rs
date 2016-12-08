extern crate sysfs_gpio;
#[macro_use]
extern crate error_chain;

mod errors;

use sysfs_gpio::{Pin, Direction};
use sysfs_gpio::Error::Unexpected;
use std::thread;
use std::time::{Duration, SystemTime};
use errors::*;

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
    pub fn read(&mut self) -> Result<Response> {

        let mut times = Vec::with_capacity(40);
        self.pin.with_exported(|| {

            // send init signal
            self.pin.set_direction(Direction::Low)?;
            self.pin.set_value(0)?;
            thread::sleep(Duration::from_millis(18));
            self.pin.set_value(1)?;
            thread::sleep(Duration::new(0, 40_000));

            // getting sensor data
            self.pin.set_direction(Direction::In)?;
            self.next_period_ns(80_000, 80_000)?; // init response
            for _ in 0..40 {
                times.push(self.next_period_ns(50_000, 26_000)?); // bit data
            }
            Ok(())

        })?;

        // convert times to bytes
        let mut bytes = [0u8; 5];
        for (val, ch) in bytes.iter_mut().zip(times.chunks(8)) {
            for t in ch {
                *val <<= 1;
                if *t > 100_000 { *val |= 1; }
            }
        }

        // checksum
        let sum: u16 = bytes.iter().take(4).map(|b| *b as u16).sum();
        if bytes[4] as u16 != sum & 0x00FF {
            Err("Invalid checksum".into())
        } else {
            Ok(Response {
                h_int: bytes[0],
                h_dec: bytes[1],
                t_int: bytes[2],
                t_dec: bytes[3],
            })
        }
    }

    /// Measures the duration in nanoseconds of a period with indicative low and high durations
    fn next_period_ns(&self, low_ns: u32, high_ns: u32) -> ::sysfs_gpio::Result<u32> {
        let start = SystemTime::now();
        let mut val = 0u8;
        let mut counter = 0u8;
        for dt in [low_ns, high_ns].iter() {
            // make sure we're at same value at half semi-period
            thread::sleep(Duration::new(0, dt / 2));
            if self.pin.get_value()? != val {
                return Err(Unexpected(format!("Expecting {} value", val)));
            }
            thread::sleep(Duration::new(0, dt / 2));
            let target = 1 - val;
            val = self.pin.get_value()?;
            while val != target {
                thread::sleep(Duration::new(0, 1000));
                counter += 1;
                if counter == 255 { // a period cannot exceed 255 microseconds
                    return Err(Unexpected("Timeout".to_string()));
                }
                val = self.pin.get_value()?;
            }
        }
        Ok(start.elapsed().unwrap().subsec_nanos())
    }

}

pub struct Response {
    h_int: u8,
    h_dec: u8,
    t_int: u8,
    t_dec: u8,
}

impl Response {
    pub fn get_temperature(&self) -> f32 {
        self.t_int as f32 + self.t_dec as f32 / 1000.
    }
    pub fn get_humidity(&self) -> f32 {
        self.h_int as f32 + self.h_dec as f32 / 1000.
    }
}
