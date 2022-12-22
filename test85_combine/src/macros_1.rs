use combine::error::ParseError;
use combine::parser::char::{char, letter, spaces};
use combine::stream::Stream;
use combine::{between, choice, many1, parser, sep_by, Parser};

/// Enum с различными типами переменных в выражении
#[derive(Debug, PartialEq)]
pub enum Expr {
    /// Какой-то идентификатор
    Id(String),

    /// Массив других выражений
    Array(Vec<Expr>),

    /// Пара выражений
    Pair(Box<Expr>, Box<Expr>),
}

/// Возврат `impl Parser` может быть использован для того, чтобы создавать переиспользуемые парсеры
fn expr_outer<Input>() -> impl Parser<Input, Output = Expr>
where
    // Вход должен быть потоком символов
    Input: Stream<Token = char>,
    // Ошибка у входного потока должна быть ошибкой парсинга
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
    // Интересное ограничение для указания возможности конверации ошибки из нашего типа
    // <Input::Error as ParseError<Input::Token, Input::Range, Input::Position>>::StreamError:
    //     From<()>,
{
    // Словом является последовательность из 1 и более букв
    let word = many1(letter());

    // Парсер пропускает прошедшие пробелы,
    //
    // Так как нам не интересно, что наш анализатор выражений
    // может принимать дополнительные пробелы между токенами, мы опускаем ошибки.
    //
    // Silent заставляет пропускать ошибки если не отработал парсер
    let skip_spaces = || spaces().silent();

    // Создает парсер, который парсит символ и пропускает все оставшиеся пробелы
    let lex_char = |c| char(c).skip(skip_spaces());

    // Рекурсивые выражения, разделенные запятыми
    let comma_list = sep_by(expr(), lex_char(','));

    // Массив состоит из выражений, разделенных запятыми
    let array = between(lex_char('['), lex_char(']'), comma_list);

    // Мы можем использовать туплы чтобы запускать отдельные парасеры в виде последовательности.
    // Конечный тип - это тупл, состоящий из вывода каждого парсера.
    // Пример: `(выражение, выражение)`
    let pair = (lex_char('('), expr(), lex_char(','), expr(), lex_char(')'))
        .map(|t| Expr::Pair(Box::new(t.1), Box::new(t.3)));

    // Оператор choise принимает тупл, слайс или массив других парсеров
    // Затем, последовательно пытается, применить к каждому вызову.
    // Здесь мы выбираем либо отдельное слово и превращаем в id, либо массив - превращаем в массив, либо пару
    // Затем, пропускаем остаточные пробелы
    choice((word.map(Expr::Id), array.map(Expr::Array), pair)).skip(skip_spaces())
}

// Так как данный парсер выражений требует рекурсивных вызовоы
// `impl Parser` вариант выше не может быть использован как есть
// так как генерируемый тип будет бесконечного размера на стеке.
//
// Мы можем избежать этого используя парсер макрос, который стирает
// внутренний тип и размер исходного типа, который мы хотим использовать рекурсивно.
//
// Данный макрос не использует `impl Trait`, что значит - он может использован быть в
// rust < 1.26 для эмуляции `impl Trait`
//
// Здесь квадратные скобки - это шаблонный параметр судя по всему
parser! {
    fn expr[Input]()(Input) -> Expr
    where [
        // Тип входного потока
        Input: Stream<Token = char>,
        
        // Ошибка у входного потока должна быть ошибкой парсинга
        Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,

        // Интересное ограничение для указания возможности конверации ошибки из нашего типа
        // <Input::Error as ParseError<Input::Token, Input::Range, Input::Position>>::StreamError:
        //     From<()>,
    ]
    {
        expr_outer()
    }
}

pub fn test_macros_1() {
    // Создаем парсер
    let mut parser = expr();

    // Пробуем распарсить текст
    let result = parser.parse("[[], (hello, world), [rust]]");

    // Конечное ожидаемое выражение
    let expr = Expr::Array(vec![
        Expr::Array(Vec::new()),
        Expr::Pair(
            Box::new(Expr::Id("hello".to_string())),
            Box::new(Expr::Id("world".to_string())),
        ),
        Expr::Array(vec![Expr::Id("rust".to_string())]),
    ]);

    assert_eq!(result, Ok((expr, "")));
}
