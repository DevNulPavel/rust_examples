use combine::{
    error::ParseError,
    parser::char::digit,
    parser::range::take_while1,
    RangeStreamOnce, {any, choice, from_str, many1, parser as parser_macro, EasyParser, Parser},
};

///////////////////////////////////////////////////////////////////////////////////////////////////

parser_macro! {
    /// `[Input]` представляет собой обычный типовой параметр (шаблонный) и описание лайфтайма
    /// для функции.
    ///
    /// Он рассахаривается в шаблонный параметр Input.
    fn integer[Input]()(Input) -> i32
    where [
        // Входной поток - стрим из символов char
        Input: RangeStreamOnce<Token = char>,
        // Ограничения на Range
        <Input as combine::StreamOnce>::Range: combine::stream::Range,
        <Input as combine::StreamOnce>::Range: combine::parser::combinator::StrLike,
        // Ошибка - реализует ошибку парсинга c типами из входа
        Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
        // Интересное ограничение для указания возможности конверации ошибки из нашего типа
        <Input::Error as ParseError<Input::Token, Input::Range, Input::Position>>::StreamError:
            From<::std::num::ParseIntError>,
    ]
    {
        // Тело должно быть блоком ( `{ <block body> }`), который заканчивается
        // выражением, которое выполняет парсин
        // let digits = many1::<String, _, _>(digit());
        // from_str(digits)

        let digits = take_while1(|c: char| c.is_ascii_digit());
        from_str(digits)
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

/// Результатом работы у нас является строка или int
#[derive(Debug, PartialEq)]
pub enum IntOrString {
    Int(i32),
    String(String),
}

///////////////////////////////////////////////////////////////////////////////////////////////////

// Указываем префикс pub для того, чтобы сказать, что парсер публичный
parser_macro! {
    /// Парсим либо интеджер, либо строку
    pub fn integer_or_string[Input]()(Input) -> IntOrString
    where [
        // Входной поток - стрим из символов char
        Input: RangeStreamOnce<Token = char>,
        // Ограничения на Range
        <Input as combine::StreamOnce>::Range: combine::stream::Range,
        <Input as combine::StreamOnce>::Range: combine::parser::combinator::StrLike,
        // Ошибка - реализует ошибку парсинга c типами из входа
        Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
        // Интересное ограничение для указания возможности конверации ошибки из нашего типа
        <Input::Error as ParseError<Input::Token, Input::Range, Input::Position>>::StreamError:
            From<::std::num::ParseIntError>,
    ]
    {
        // Парсин инта, используем парсер выше
        let int_parser = integer().map(IntOrString::Int);

        // Парсер для строки
        let str_parser = many1(any()).map(IntOrString::String);

        // Макрос, который внутри просто делает обертку .or
        choice!(
            int_parser,
            str_parser
        )
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

parser_macro! {
    // Даем создаваемому типу новое имя
    #[derive(Clone)]
    pub struct Twice;


    /// Специальный парсер, генерирующий последовательность одинаковых парсеров
    pub fn twice[Input, F, P](f: F)(Input) -> (P::Output, P::Output)
        where [
            // Парсер со входом
            P: Parser<Input>,
            // Функтор, генерирующий парсеры
            F: FnMut() -> P
        ]
    {
        (f(), f())
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////

pub fn test_macros_2() {
    // Проверяем парсер интовых значений
    assert_eq!(integer().easy_parse("123"), Ok((123, "")));
    assert!(integer().easy_parse("!").is_err());

    // Проверяем парсер интовых или строковых значений
    assert_eq!(
        integer_or_string().easy_parse("123"),
        Ok((IntOrString::Int(123), ""))
    );
    assert_eq!(
        integer_or_string().easy_parse("abc"),
        Ok((IntOrString::String("abc".to_string()), ""))
    );

    // Создаем дублирующий парсер для отдельных чисел
    assert_eq!(twice(digit).parse("123"), Ok((('1', '2'), "3")));
}
