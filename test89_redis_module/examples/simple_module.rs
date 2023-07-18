use redis_module::{redis_module, Context, RedisError, RedisResult, RedisString};

/////////////////////////////////////////////////////////////////////////////////

/// Непосредственно функция-обработчик вызова команды.
/// Занимается тем, что перемножает переданные числа команде и выдает результат.
fn hello_mul(_: &Context, args: Vec<RedisString>) -> RedisResult {
    // Сначала проверяем количество аргументов
    if args.len() < 2 {
        // Если аргументов недостаточно - выходим с ошибкой
        return Err(RedisError::WrongArity);
    }

    // Результат умножения значений.
    // Стандартное значение 1, так как мы перемножаем значения.
    let mut result = 1;

    // Пропускаем первый аргумент, так как это сама команда.
    let nums_iter = args
        .into_iter()
        .skip(1);

    // Аргументы перебираем один за другим
    for arg in nums_iter.into_iter() {
        // Парсим в int
        let int_value = arg.parse_integer()?;

        // Домножаем
        result *= int_value;
    }

    // Конвертируем в результат
    Ok(result.into())
}

/////////////////////////////////////////////////////////////////////////////////

// Регистрация данного модуля в системе
// Описание макроса:
// https://docs.rs/redis-module/2.0.4/redis_module/macro.redis_module.html
redis_module! {
    // Имя данного модуля
    name: "hello",
    // Версия самого модуля
    version: 1,
    // Используемый аллокатор. Формат: (тип, код создания)
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    // Кастомные наши типы
    data_types: [],
    // Регистрируемые команды редиса.
    // Формат: [имя, вызываемая функция, флаги, первый ключ, последний ключ, шаг]
    // Флаги можно посмотреть здесь поиском по `RedisModule_CreateCommand`: 
    // https://redis.io/docs/reference/modules/modules-api-ref/
    commands: [
        ["hello.mul", hello_mul, "", 0, 0, 0],
    ],
}
