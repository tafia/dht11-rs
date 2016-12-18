extern crate dht11;

use std::time::Instant;
use dht11::delay_us;

fn main() {
    let s = Instant::now();
    delay_us(10);
    println!("{}", s.elapsed().subsec_nanos());
}
