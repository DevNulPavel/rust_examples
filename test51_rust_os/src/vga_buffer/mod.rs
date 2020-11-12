mod buffer;
mod color_code;
mod color;
mod writer;
mod screen_char;
#[macro_use] mod print;

use core::{
    fmt::{
        Write
    }
};
use self::{
    writer::{
        WRITER
    }
};

pub fn test_print_something() {
    let mut writer = WRITER.lock();
    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
    writer.write_byte(b'\n');
    write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap();
    println!("Another test string");
}
