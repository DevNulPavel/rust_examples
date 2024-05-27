use volatile::{
    Volatile
};
use super::{
    screen_char::{
        ScreenChar
    }
};


pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
pub struct Buffer {
    // Используем Volatile чтобы компилятор не оптимизировал наш код так как он не читается нигде, а только пишется
    pub(crate) chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}