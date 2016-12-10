extern crate wiringpi;

use wiringpi::pin::{WiringPi, InputPin};
use wiringpi::pin::Value::{self, Low, High};
use wiringpi::time::{delay, delay_microseconds};

#[derive(Debug)]
pub enum Error {
    TimeOut,
    CheckSum,
}

pub type Result<T> = ::std::result::Result<T, Error>;

pub struct DHT11 {
    pi: wiringpi::WiringPi<WiringPi>,
    pin: u16,
}

impl DHT11 {

    /// Initialize a new connection on pin with DHT11 sensor
    pub fn new(pin_num: u16) -> DHT11 {
        DHT11 {
            pi: wiringpi::setup(),
            pin: pin_num,
        }
    }

    /// Reads humidity and temperature
    pub fn read(&mut self) -> Result<Measures> {

        // send init request
        {
            let output = self.pi.output_pin(self.pin);
            output.digital_write(Low);
            delay(18);
            output.digital_write(High);
            delay_microseconds(40);
        }

        // get data from sensor
        let mut bytes = [0u8; 5];
        {
            let input = self.pi.input_pin(self.pin);
            wait_level(&input, Low)?;
            wait_level(&input, High)?;
            wait_level(&input, Low)?;
            for b in bytes.iter_mut() {
                for _ in 0..8 {
                    *b <<= 1;
                    wait_level(&input, High)?;
                    let dur = wait_level(&input, Low)?;
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

fn wait_level(pin: &InputPin<WiringPi>, level: Value) -> Result<u8> {
    for i in 0u8..255 {
        if pin.digital_read() == level { return Ok(i); }
        delay_microseconds(1);
    }
    Err(Error::TimeOut)
}

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
