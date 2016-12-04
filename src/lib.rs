extern crate sysfs_gpio;
#[macro_use]
extern crate error_chain;

mod errors;

use sysfs_gpio::{Pin, PinPoller, Direction, Edge};
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

    pub fn read(&mut self) -> Result<Response> {
        self.pin.set_edge(Edge::BothEdges)?;
        let mut poller = self.pin.get_poller()?;

        // send init signal
        self.pin.set_direction(Direction::Out)?;
        self.pin.set_value(0)?;
        thread::sleep(Duration::from_millis(18));
        self.pin.set_value(1)?;
        thread::sleep(Duration::new(0, 40000)); // wait 40us

        // getting init response
        self.pin.set_direction(Direction::In)?;
        for _ in 0..3 {
            if poller.poll(1)?.is_none() {
                return Err("Initialization failed, device did not answer on time".into());
            }
        }

        // getting sensor data
        let mut bytes = [0u8; 5];
        for b in &mut bytes {
            *b = read_byte(&mut poller)?;
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

fn read_byte(poller: &mut PinPoller) -> Result<u8> {
    let mut val = 0u8;
    for _ in 0..8 {
        val <<= 1;
        let start = SystemTime::now();
        if poller.poll(1)?.is_none() || start.elapsed()?.subsec_nanos() > 60000 {
            return Err("Cannot get value, device did not answer on time".into());
        }
        match poller.poll(1)? {
            None => return Err("Cannot get value, device did not answer on time".into()),
            Some(_) => match start.elapsed()?.subsec_nanos() {
                70000...85000 => (), // false (should be 76us to 78us), do nothing
                110000...130000 => val |= 1, // true, should be 120us
                _ => return Err("Cannot get value, device did not answer on time".into()),
            }
        }
    }
    Ok(val)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
