#![warn(clippy::all)]

mod fibonacci;

use fibonacci::fibonacci_recursive;

fn main() {
    let val = fibonacci_recursive(10);
    println!("{}", val);
}
