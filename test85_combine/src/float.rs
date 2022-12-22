use combine::{
    error::UnexpectedParse,
    parser::{
        choice::optional,
        range::{recognize, take_while, take_while1},
        token::token,
    },
    Parser,
};

/*/// В генерируемом парсере указываем сразу же нужные нам типы данных
fn new_parser<'a>() -> impl Parser<&'a [u8], Output = f32> {
    // Первая часть цифр, сами символы не забираем из потока, а лишь проверяем,
    // что есть хотя бы одна цифра
    let first_digits = skip_many1(digit());

    // Вторая часть состоит из двух токенов последовательных - сначала точка, потом пропускаемые цифры
    // как в первой части.
    // Но здесь точка является обязательным символом.
    let second_digits_tuple = (token(b'.'), skip_many(digit()));

    // Вторая часть цифр у нас опциональная, поэтому точка была обязательна
    // Если значения есть в парсинге, то возвращаем результат, если нет - None
    let second_digits_option = optional(second_digits_tuple);

    // Цепочка из первых символов, затем - второй части с точкой
    let float_seq = (first_digits, second_digits_option);

    // Обертка recognize нужна для того, чтобы просто получить соответствие всему патетрну, но без потребления парсером
    let parser = recognize(float_seq);

    // После этого - пытаемся сконвертировать в utf8 + парсим f64
    parser.and_then(|bs: &[u8]| {
        // У нас точно валидные символы - можем использовать unsafe
        let s = unsafe { std::str::from_utf8_unchecked(bs) };

        // Парсим флоат
        s.parse::<f32>().map_err(|_| UnexpectedParse::Unexpected)
    })
}*/

/// В генерируемом парсере указываем сразу же нужные нам типы данных
fn new_parser<'a>() -> impl Parser<&'a [u8], Output = f32> {
    // Первая часть цифр, сами символы не забираем из потока, а лишь проверяем,
    // что есть хотя бы одна цифра
    let first_digits = take_while1(|v: u8| v.is_ascii_digit());

    // Вторая часть состоит из двух токенов последовательных - сначала точка, потом пропускаемые цифры
    // как в первой части.
    // Но здесь точка является обязательным символом.
    let second_digits_tuple = (token(b'.'), take_while(|v: u8| v.is_ascii_digit()));

    // Вторая часть цифр у нас опциональная, поэтому точка была обязательна
    // Если значения есть в парсинге, то возвращаем результат, если нет - None
    let second_digits_option = optional(second_digits_tuple);

    // Цепочка из первых символов, затем - второй части с точкой
    let float_seq = (first_digits, second_digits_option);

    // Обертка recognize нужна для того, чтобы просто получить соответствие всему патетрну в виде tuple
    let parser = recognize(float_seq);

    // После этого - пытаемся сконвертировать в utf8 + парсим f64
    parser.and_then(|bs: &[u8]| {
        // У нас точно валидные символы - можем использовать unsafe
        let s = unsafe { std::str::from_utf8_unchecked(bs) };

        // Парсим флоат
        s.parse::<f32>().map_err(|_| UnexpectedParse::Unexpected)
    })
}

pub fn float() {
    {
        let mut parser = new_parser();

        // Парсим
        let result = parser.parse(b"123.45".as_slice());

        // Сравниваем результат
        assert_eq!(result, Ok((123.45_f32, b"".as_slice())));
    }

    {
        let mut parser = new_parser();

        // Парсим
        let result = parser.parse(b"123.".as_slice());

        // Сравниваем результат
        assert_eq!(result, Ok((123.0_f32, b"".as_slice())));
    }

    {
        let mut parser = new_parser();

        // Парсим
        let result = parser.parse(b"123".as_slice());

        // Сравниваем результат
        assert_eq!(result, Ok((123.0_f32, b"".as_slice())));
    }
}
