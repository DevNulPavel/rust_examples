use core::{
    fmt::{
        self,
        Write
    }
};
use spin::{
    Mutex
};
use lazy_static::{
    lazy_static
};
use super::{
    color_code::{
        ColorCode
    },
    color::{
        Color
    },
    buffer::{
        Buffer,
        BUFFER_HEIGHT,
        BUFFER_WIDTH
    },
    screen_char::{
        ScreenChar
    }
};

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

pub struct Writer {
    pub(super) column_position: usize,
    pub(super) color_code: ColorCode,
    // Буффер у нас всегда активен, так то никаких проблем со статическим временем жизни
    pub(super) buffer: &'static mut Buffer
}

impl Writer {
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // Отсеиваем верные значения байтов
                // @ нужна для получения конкретного значения, но в целом - можно и убрать
                val @ 0x20..=0x7e | val @ b'\n' => {
                    self.write_byte(val);
                },
                // Если не подходит символ - тогда просто фигню пишем в виде символа ~
                _ => {
                    self.write_byte(0xfe)
                }
            }

        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            // Если у нас новая строка
            b'\n' => {
                self.new_line();
            },
            // Байт обычный
            byte => {
                // Текущая позиция писателя больше или равно ширине строки
                if self.column_position >= BUFFER_WIDTH {
                    // Тогда переходим на новую строку
                    self.new_line();
                }

                // Пока пишем в самую нижнюю строку
                let row = BUFFER_HEIGHT - 1;
                // Позиция по горизонтали
                let col = self.column_position;

                // Получаем цвет
                let color_code = self.color_code;

                // Пишем в буффер символ
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                
                // Смещаемся
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self) {
        // Проходим все строки, верхнее число исключается, иначе надо писать ..=
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                // Записываем в символ предыдущей строки текущий символ
                let character = self.buffer.chars[row][col].read();
                self.buffer
                    .chars[row - 1][col]
                    .write(character);
            }
        }
        // Чистим самую нижнюю строку
        self.clear_row(BUFFER_HEIGHT - 1);
        // Обнуляем позицию в строке на самое начало
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        // Зазовый пустой символ, реализуется трейт Copy, поэтому можем копировать
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer
                .chars[row][col]
                .write(blank);
        } 
    }
}

// Реализация поддержки форматированного вывода в данный объект
impl Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}