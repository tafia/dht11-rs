extern crate sysfs_gpio;
#[macro_use]
extern crate error_chain;

mod errors;

use sysfs_gpio::{Pin, Direction, Edge};
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

        let mut times = Vec::with_capacity(41);
        self.pin.with_exported(|| {

            // send init signal
            self.pin.set_direction(Direction::Low)?;
            self.pin.set_value(0)?;
            thread::sleep(Duration::from_millis(18));
            self.pin.set_value(1)?;
            thread::sleep(Duration::new(0, 40_000));

            // getting sensor data
            let start = SystemTime::now();
            let mut poller = self.pin.get_poller()?;
            self.pin.set_direction(Direction::In)?;
            self.pin.set_edge(Edge::FallingEdge)?;
            while poller.poll(1)?.is_some() {
                times.push(start.elapsed().unwrap());
            }
            Ok(())
        })?;

        // convert times to bytes
        let mut intervals = times.windows(2).map(|w| w[1] - w[0]);
        let mut bytes = [0u8; 5];
        for val in bytes.iter_mut() {
            for i in intervals.by_ref().take(8) {
                *val <<= 1;
                if i > Duration::new(0, 80_000) { *val |= 1; }
            }
        }

        // checksum
        // let sum: u16 = bytes.iter().take(4).map(|b| *b as u16).sum();
        // if bytes[4] as u16 != sum & 0x00FF {
        //     Err("Invalid checksum".into())
        // } else {
            Ok(Response {
                h_int: bytes[0],
                h_dec: bytes[1],
                t_int: bytes[2],
                t_dec: bytes[3],
            })
        //}
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
