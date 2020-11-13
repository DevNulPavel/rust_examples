mod buffer;
mod color_code;
mod color;
pub mod writer;
mod screen_char;
#[macro_use] pub mod print;

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
    WRITER.lock().write_byte(b'H');
    WRITER.lock().write_string("ello ");
    WRITER.lock().write_string("Wörld!");
    WRITER.lock().write_byte(b'\n');
    WRITER.lock().write_str("asdsd\n").unwrap();
    // let args = format_args!("TEST: ");
    // let args_str = args.as_str().unwrap();
    write!(WRITER.lock(), "The numbers are {} and {}\n", 42, 1.0/3.0).unwrap();
    // write!(WRITER.lock(), "Args: {}\n", args_str).unwrap();
    // print::_print(args);
    println!("Another test string");
}
