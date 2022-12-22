use combine::{
    error::Commit,
    parser::char::{digit, letter},
    Parser, StdParseResult,
};

fn test(input: &mut &'static str) -> StdParseResult<(char, char), &'static str> {
    // Парсер для отдельных цифр
    let mut p = digit();

    // Создаем парсер, где цепочкой идет цифра + буква потом
    // Сразу же парсим значение, конвертируем в результат с ошибкой в виде строки
    // Первая часть - это результат парсинга
    // Вторая - тип Commited, которая говорит смогли распарсить или нет
    let ((digit_1, _letter_1), committed) =
        (p.by_ref(), letter()).parse_stream(input).into_result()?;

    // Парсим дальше то, что можем если не смогли распарсить первую часть
    let (digit_2, committed) = committed.combine(|_| p.parse_stream(input).into_result())?;

    Ok(((digit_1, digit_2), committed))
}

pub fn rest_ref() {
    // Входная строка байт
    let mut input = "1a23";

    let res = test(&mut input).map(|(t, c)| (t, c.map(|_| input)));

    assert_eq!(res, Ok((('1', '2'), Commit::Commit("3"))));
}
