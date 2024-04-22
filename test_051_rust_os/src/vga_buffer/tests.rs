use core::{
    fmt::{
        Write
    }
};
use super::{
    writer::{
        WRITER
    },
    buffer::{
        BUFFER_HEIGHT
    }
};

#[test_case]
fn test_println_simple() {
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

#[test_case]
fn test_println_many() {
    for _ in 0..200 {
        println!("test_println_many output");
    }
}

#[test_case]
fn test_println_output() {
    let s = "Some test string that fits on a single line";
    println!("{}", s);
    // Обходим строку, пишем в буффер экрана, проверяем что все ок
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}