mod buffer;
mod color_code;
mod color;
mod writer;
mod screen_char;

use core::{
    fmt::{
        Write
    }
};
use self::{
    writer::{
        Writer
    },
    color::{
        Color
    },
    buffer::{
        Buffer
    },
    color_code::{
        ColorCode
    }
};

pub fn test_print_something() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
    writer.write_byte(b' ');
    write!(writer, "The numbers are {} and {}", 42, 1.0/3.0).unwrap()
}