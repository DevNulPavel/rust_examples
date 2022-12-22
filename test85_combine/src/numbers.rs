use combine::{
    attempt,
    parser::{
        char::{char, spaces},
        range::take_while1,
    },
    sep_by,
    stream::easy,
    EasyParser, Parser,
};

pub fn parse_numbers() {
    // Берем одно или более число, собираем все в строку
    // Затем парсим число с помощью метода parse + map
    // Особенность many в том, что буфферизует в какой-то буффер
    // take_while1 позволяет не буфферизировать значения, работать со слайсом
    let number =
        take_while1(|v: char| v.is_numeric()).map(|string: &str| string.parse::<i32>().unwrap());

    // Сепаратором являются разные виды пробелов от нуля и более
    // Запятая должна быть обязательно в центре
    // Обертка attempt позволяет не забирать элементы, если следующий парсер падает, а просто прекратить работу
    let separator = attempt(spaces().and(char(',')).and(spaces()));

    // Парсим интенджеры, разделенные запятыми, пропуская пробелы
    let mut parser = sep_by(number, separator);

    // Данные парсинга
    let input = "1234, 45, 78 ,123 ,  1234, 123 asd as";

    // Парсим
    // easy_parse принимать может совершенно разные типы, даже стрим данных с определенными манипуляциями
    // Возвращает результаты парсинга и оставшуюся часть
    let result: Result<(Vec<i32>, &str), easy::ParseError<&str>> = parser.easy_parse(input);
    match result {
        Ok((value, remain)) => {
            println!("Values: {:?}, remain: '{}'", value, remain);
        }
        Err(err) => {
            eprintln!("Error: {}", err);
        }
    }
}
