extern crate cupi;

use std::time::{Instant, Duration};
use cupi::{CuPi, PinInput, delay_ms, DigitalWrite, DigitalRead};
use cupi::Logic::{self, Low, High};

#[derive(Debug)]
pub enum Error {
    Cupi(cupi::Error),
    TimeOut,
    CheckSum,
}

impl From<cupi::Error> for Error {
    fn from(e: cupi::Error) -> Error {
        Error::Cupi(e)
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub struct DHT11 {
    pin: cupi::PinOptions,
}

impl DHT11 {

    /// Initialize a new connection on pin with DHT11 sensor
    pub fn new(pin_num: usize) -> Result<DHT11> {
        let cupi = CuPi::new()?;
        Ok(DHT11 {
            pin: cupi.pin(pin_num)?
        })
    }

    /// Reads humidity and temperature
    pub fn read(&mut self) -> Result<Measures> {

        // send init request
        {
            let mut output = self.pin.high().output();
            output.digital_write(Low)?;
            delay_ms(18);
            output.digital_write(High)?;
            delay_us(40);
        }

        // get data from sensor
        let mut bytes = [0u8; 5];
        {
            let mut input = self.pin.input();
            wait_level(&mut input, Low)?;
            wait_level(&mut input, High)?;
            wait_level(&mut input, Low)?;
            for b in bytes.iter_mut() {
                for _ in 0..8 {
                    *b <<= 1;
                    wait_level(&mut input, High)?;
                    let dur = wait_level(&mut input, Low)?;
                    if dur > 16 { *b |= 1; }
                }
            }
        }

        let sum: u16 = bytes.iter().take(4).map(|b| *b as u16).sum();
        if bytes[4] as u16 == sum & 0x00FF {
            Ok(Measures { 
                temperature: bytes[2],
                humidity: bytes[0],
            })
        } else {
            Err(Error::CheckSum)
        }
    }

}

fn wait_level(pin: &mut PinInput, level: Logic) -> Result<u8> {
    for i in 0u8..255 {
        if pin.digital_read()? == level { return Ok(i); }
        delay_us(1);
    }
    Err(Error::TimeOut)
}

/// Measures of temperature and humidity
///
/// Given dht11 precision both are simple `u8`
pub struct Measures {
    temperature: u8,
    humidity: u8,
}

impl Measures {
    pub fn get_temperature(&self) -> u8 {
        self.temperature
    }
    pub fn get_humidity(&self) -> u8 {
        self.humidity
    }
}

/// Sleep for `us` microseconds
///
/// Cannot use regular `thread::sleep` below 100ms
pub fn delay_us(us: u32) {
    let target = Instant::now() + Duration::new(0, us * 1000);
    while Instant::now() < target { }
}
