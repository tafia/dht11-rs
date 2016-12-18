extern crate dht11;

use dht11::DHT11;
use std::thread;
use std::time::Duration;
use std::env;

fn main() {
    let pin = env::args().skip(1).next()
        .expect("Please precise pin number")
        .parse()
        .expect("Pin number must be an integer");
    //let mut sensor = DHT11::new(pin);
    let mut sensor = DHT11::new(pin).unwrap();
    loop {
        match sensor.read() {
            Ok(r) => println!("Temperature: {}, Humidity: {}", 
                              r.get_temperature(), r.get_humidity()),
            Err(e) => println!("{:?}", e),
        }
        thread::sleep(Duration::from_millis(1000));
    }
}
