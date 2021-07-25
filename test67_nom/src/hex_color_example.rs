use eyre::Context;
use log::debug;
use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    sequence::tuple
};

#[derive(Debug, PartialEq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

/// Парсим строку как шестнадцетиричное число
fn hex_primary(input: &str) -> nom::IResult<&str, u8> {
    // Это шестнадцатиричное число?
    fn is_hex_digit(c: char) -> bool {
        c.is_digit(16)
    }

    // Из шестнадцетиричного строкового числа в число
    fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
        u8::from_str_radix(input, 16)
    }

    // take_while_m_n - принимать от 2 до 2х символов, удовлетворяющих правилу
    // map_res - конвертирует результат в другой тип
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

/// Парсим шестнадцатиричное число
fn parse_hex_color(input: &str) -> nom::IResult<&str, Color> {
    // Создаем функцию, которая ищет # и выдает наружу новый вывод + сам символ
    let (input, tag_output) = tag("#")(input)?;
    debug!("New input: {}, found: {}", input, tag_output);

    // Создаем функцию парсинга из четрых элементов, где каждый элемент - это шестнастиричное число
    let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;
    debug!("Tag input: {}, output: {:?}", input, (red, green, blue));

    Ok((input, Color { red, green, blue }))
}

pub fn test_parse_hex_color() -> Result<(), eyre::Error> {
    let (input, color) = parse_hex_color("#2F14DF").wrap_err("Color parse failed")?;

    eyre::ensure!(input == "", "New input must be empty");
    #[rustfmt::skip]
    eyre::ensure!(color == Color{red: 47, green: 20, blue: 223}, "Color must be valid");

    Ok(())
}
