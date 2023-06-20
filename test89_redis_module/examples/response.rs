use redis_module::{
    redis_module, redisvalue::RedisValueKey, Context, NextArg, RedisError, RedisResult,
    RedisString, RedisValue,
};
use std::collections::{HashMap, HashSet};

/// Функция для извлечения нужных полей из хешмапы с возвратом в виде ключ:значение
fn map_mget(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    // Проверяем количество аргументов
    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    // Убираем первый аргумент, а именно саму команду
    let mut args = args.into_iter().skip(1);

    // Получаем имя ключа из итератора и продвигаем итератор далее
    let key_name = args.next_arg()?;

    // Все те поля, которые мы хотим получить из хешмапы
    // TODO: SmallVec
    let field_names: Vec<RedisString> = args.collect();

    // Пытаемся открыть значение по определенному ключу
    let key = ctx.open_key(&key_name);

    // Пытаемся получить все значения из хешмапы с нужными ключами
    let values = key.hash_get_multi(&field_names)?;

    // Смотрим что там нам доступно
    let res = match values {
        // Какие-то начения по данным ключам у нас есть в хешмапе
        Some(values) => {
            // Создаем локальную хешмапу для значений c максимальным размером
            let mut map: HashMap<RedisValueKey, RedisValue> =
                HashMap::with_capacity(field_names.len());

            // Перебираем прилетевшие значения, но все ли они там будут для запрашиваемых полей?
            for (field, value) in values.into_iter() {
                // Сохраняем в конечную хешмапу
                map.insert(
                    RedisValueKey::BulkRedisString(field),
                    RedisValue::BulkRedisString(value),
                );
            }

            // Возвращаем результат
            RedisValue::Map(map)
        }

        // Не смогли получить значения для ключей из хешмапы
        None => RedisValue::Null,
    };

    Ok(res)
}

fn map_unique(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    // Проверяем количество аргументов
    if args.len() < 2 {
        return Err(RedisError::WrongArity);
    }

    // Убираем первый аргумент, а именно саму команду
    let mut args = args.into_iter().skip(1);

    // Получаем имя ключа из итератора и продвигаем итератор далее
    let key_name = args.next_arg()?;

    // Получаем нужные поля для извлечения из хешмапы
    // TODO: SmallVec
    let field_names: Vec<RedisString> = args.collect();

    // Открываем на чтение определенный ключ
    let key = ctx.open_key(&key_name);

    // Пытаемся получить все значения из хешмапы с нужными ключами
    let values = key.hash_get_multi(&field_names)?;

    // Смотрим что там нам прилетело
    let res = match values {
        None => RedisValue::Null,
        Some(values) => {
            // Создаем сет для накопления лишь уникальных значений из хешмап
            let mut set: HashSet<RedisValueKey> = HashSet::with_capacity(field_names.len());

            // Перебираем элементы и записываем в сет
            for (_, value) in values.into_iter() {
                set.insert(RedisValueKey::BulkRedisString(value));
            }

            // Вернем сет наружу
            RedisValue::Set(set)
        }
    };

    Ok(res)
}

//////////////////////////////////////////////////////

redis_module! {
    name: "response",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [],
    // Регистрируемые команды редиса.
    // Формат: [имя, вызываемая функция, флаги, первый ключ, последний ключ, шаг]
    // Флаги можно посмотреть здесь поиском по `RedisModule_CreateCommand`: 
    // https://redis.io/docs/reference/modules/modules-api-ref/
    commands: [
        ["map.mget", map_mget, "readonly", 1, 1, 1],
        ["map.unique", map_unique, "readonly", 1, 1, 1]
    ]
}
