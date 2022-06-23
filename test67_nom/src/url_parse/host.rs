use super::error::ParsingError;
use nom::{
    branch::alt,
    bytes::complete::{tag, take, take_while},
    character::{
        complete::{alpha1, alphanumeric0, alphanumeric1, digit1},
        is_digit,
    },
    combinator::{flat_map, opt, recognize},
    error::{context, ErrorKind},
    multi::{count, many0, many1, many_m_n, separated_list1},
    sequence::{terminated, tuple},
    AsChar, InputTakeAtPosition,
};
use std::net::Ipv4Addr;

#[derive(Debug)]
pub enum HostAddr<'a> {
    Name(&'a str),
    Ip(Ipv4Addr),
}

#[derive(Debug)]
pub struct Host<'a> {
    pub addr: HostAddr<'a>,
    pub port: Option<u16>,
}

impl<'a> Host<'a> {
    pub fn try_parse(text: &str) -> Result<(&str, Host), ParsingError> {
        // DNS name
        // let dns_name_func = tuple((alphanumeric1, many0(terminated(alphanumeric1, tag("."))), alphanumeric1));

        // // Пишем комбинатор с контекстом
        // // alt((dns_name_func, ip_func))
        // let mut parse_func = context("host", alt((dns_name_func, ip_func)));

        // Парсим ввод, затем пытаемся распарсить результат
        // let (remind, result) = dns_name_func(text)?;

        todo!()

        // // Теперь уже парсим схему из строки
        // let auth = result.map(|result| Authority {
        //     username: result.0,
        //     password: result.1,
        // });

        // Ok((remind, auth))
    }
}

/// Делаем парсинг доменного имени
fn parse_dns_name(input: &str) -> Result<(&str, &str), ParsingError> {
    // Локальная функция для парсинга из alphanumeric1, но дополнительно еще добавлен символ `-`
    fn alphanumerichyphen1<T>(i: T) -> nom::IResult<T, T>
    where
        T: InputTakeAtPosition,
        <T as InputTakeAtPosition>::Item: AsChar,
    {
        i.split_at_position1_complete(
            |item| {
                let char_item = item.as_char();
                char_item != '-' && !char_item.is_alphanum()
            },
            ErrorKind::AlphaNumeric,
        )
    }

    // Функция обработки входного текста
    let mut parse_func = context(
        "host_name_dns",
        // Может быть как первый, так и второй вариант, для этого ипользуем `alt`
        recognize(alt((
            // `tuple` нужен для того, чтобы просто описать определенную последовательной правил
            tuple((many1(terminated(alphanumerichyphen1, tag("."))), alpha1)),
            // Сигнатура для alt вызовов должна совпадать, поэтому приходится делать заглушку в виде take(0)
            tuple((many_m_n(1, 1, alphanumerichyphen1), take(0_usize))),
        ))),
    );

    // Парсим значения, если второй элемент у нас не пустой, тогда добавляем в общую кучу
    Ok(parse_func(input)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dns_host_parsing_1() {
        let (reminded, parsed) = parse_dns_name("www.test.com/test/path?test=1").unwrap();
        assert_eq!(reminded, "/test/path?test=1");
        assert_eq!(parsed, "www.test.com");
    }

    #[test]
    fn dns_host_parsing_2() {
        let (reminded, parsed) = parse_dns_name("www.test-testing.com").unwrap();
        assert_eq!(reminded, "");
        assert_eq!(parsed, "www.test-testing.com");
    }

    #[test]
    fn dns_host_parsing_3() {
        let (reminded, parsed) = parse_dns_name("localhost").unwrap();
        assert_eq!(reminded, "");
        assert_eq!(parsed, "localhost");
    }

    #[test]
    fn dns_host_parsing_failed() {
        match parse_dns_name("192.168.10.1/test/path").unwrap_err() {
            ParsingError::ParsingFailed(nom::Err::Error(nom::error::Error::<_> { input, .. })) => {
                assert_eq!(input, "192.168.10.1/test/path");
            }
            err => panic!("Unexpected error: {}", err),
        }
    }
}
