#![no_std]

use core::time::Duration;
use embedded_hal::{
    digital::v2::{OutputPin, StatefulOutputPin},
    timer::CountDown,
};

const EPSILON: i32 = 5;
const LOW_LOW: i32 = 50 - EPSILON;
const LOW_HIGH: i32 = 50 + EPSILON;
const ZERO_RANGE_LOW: i32 = 26 - EPSILON;
const ZERO_RANGE_HIGH: i32 = 26 + EPSILON;
const ONE_RANGE_LOW: i32 = 70 - EPSILON;
const ONE_RANGE_HIGH: i32 = 70 + EPSILON;

#[derive(Debug)]
pub enum Error<P> {
    Ack,
    Low,
    High,
    CheckSum,
    Pin(P),
}

fn nb_err<P>(e: P) -> nb::Error<Error<P>> {
    nb::Error::Other(Error::Pin(e))
}

fn err<P>(e: nb::Error<P>) -> nb::Error<Error<P>> {
    match e {
        nb::Error::WouldBlock => nb::Error::WouldBlock,
        nb::Error::Other(p) => nb::Error::Other(Error::Pin(p)),
    }
}

/// A DHT sensor
#[derive(Debug)]
pub struct DHT;

/// A trait to represent Interrupts
pub trait Interrupt: OutputPin + StatefulOutputPin {
    /// Wait for the pin to reach corresponding level.
    /// Return the duration in microseconds since last level change
    ///
    /// Need to be able to get both high and low state, which corresponds to both raising and
    /// falling edge interrupt
    fn elapsed_us(&mut self, high: bool) -> nb::Result<i32, Self::Error>;
}

impl DHT {
    /// Initialize DHT sensor connected to `pin`
    ///
    /// `timer` must allow milliseconds countdown
    pub fn read<T, P>(mut pin: P, mut timer: T) -> nb::Result<Measure, Error<P::Error>>
    where
        T: CountDown<Time = Duration>,
        P: Interrupt,
    {
        // send init request
        pin.set_high().map_err(nb_err)?;
        pin.set_low().map_err(nb_err)?;
        timer.start(Duration::new(0, 18_000_000));
        nb::block!(timer.wait()).expect("Cannot fail");
        pin.set_high().map_err(nb_err)?;
        while !pin.is_set_high().map_err(nb_err)? {
            timer.start(Duration::new(0, 10));
            nb::block!(timer.wait()).expect("Cannot fail");
        }

        // acknowledge
        let _ = pin.elapsed_us(false).map_err(err)?;
        let ack_low = pin.elapsed_us(true).map_err(err)?;
        let ack_high = pin.elapsed_us(false).map_err(err)?;
        if (ack_low - 80).abs() > EPSILON || (ack_high - 80).abs() > EPSILON {
            return Err(nb::Error::Other(Error::Ack));
        }

        // read
        let mut data = 0u64;
        for _ in 0..40 {
            let low = pin.elapsed_us(true).map_err(err)?;
            if low < LOW_LOW || low > LOW_HIGH {
                return Err(nb::Error::Other(Error::Low));
            }
            let high = pin.elapsed_us(false).map_err(err)?;
            match high {
                ZERO_RANGE_LOW...ZERO_RANGE_HIGH => (),
                ONE_RANGE_LOW...ONE_RANGE_HIGH => data |= 1,
                _ => return Err(nb::Error::Other(Error::Low)),
            }
            data <<= 1;
        }

        // checksum
        if (data >> 8 + data >> 16 + data >> 24 + data >> 32) & 0xFF != data & 0xFF {
            return Err(nb::Error::Other(Error::CheckSum));
        }

        Ok(Measure((data >> 8) as u32))
    }
}

/// A set of Humidity and Temperature measurement
pub struct Measure(u32);

impl Measure {
    /// Get Humidity
    pub fn humidity(&self) -> f32 {
        (self.0 >> 24) as f32 + ((self.0 >> 16) & 0xFF) as f32 / 10000.
    }
    pub fn temperature(&self) -> f32 {
        ((self.0 >> 8) & 0xFF) as f32 + (self.0 & 0xFF) as f32 / 10000.
    }
}
