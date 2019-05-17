extern crate dht11;

use dht11::delay_us;
use std::time::Instant;

fn main() {
    let s = Instant::now();
    delay_us(10);
    println!("{}", s.elapsed().subsec_nanos());
}
