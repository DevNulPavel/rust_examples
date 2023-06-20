use redis_module::{
    key::RedisKey, redis_module, Context, KeyType, KeysCursor, RedisResult, RedisString,
    RedisValue, Status,
};
use smallstr::SmallString;
use std::fmt::Write;

fn init(ctx: &Context, _args: &[RedisString]) -> Status {
    // Так можно выводить логи
    let Ok(redis_version) = ctx.get_redis_version() else {
        ctx.log_warning("Vercion receive failed");
        return Status::Err;
    };

    ctx.log_notice(
        format!(
            "Redis version: {}.{}.{}",
            redis_version.major, redis_version.minor, redis_version.patch
        )
        .as_str(),
    );

    Status::Ok
}

fn deinit(ctx: &Context) -> Status {
    ctx.log_warning("Module deinit");

    Status::Ok
}

fn scan_keys(ctx: &Context, _args: Vec<RedisString>) -> RedisResult {
    // Создаем курсор для сохранения позиции обхода всех ключей в редисе
    let cursor = KeysCursor::new();

    // Буфер для результатов
    let mut res = Vec::new();

    // Специальный колбек, который вызывается при полном переборе всех ключей.
    // Первый параметр - это непосредственно ключ, второй - собственно, само значение в редисе?
    let scan_callback = |ctx: &Context, key_name: RedisString, key: Option<&RedisKey>| {
        // Получим тип из key
        let key_type = key.map(|v| v.key_type()).unwrap_or(KeyType::Empty);

        // Выведем отладочную инфу в строку, unwrap можно, пишем в оперативку.
        let mut info_string: SmallString<[u8; 64]> = SmallString::<[u8; 64]>::new();
        write!(
            &mut info_string,
            "key_name: '{}', key_type: '{:?}'",
            key_name, key_type
        )
        .unwrap();

        // Строку Redis из буфера на стеке в контексте редиса
        let redis_string = ctx.create_string(info_string.as_str());

        // Сохраняем в буфер с результатом
        res.push(RedisValue::BulkRedisString(redis_string));

        // Либо можем не заниматься конвертацией и еспользовать сразу redis типы
        // RedisValue::StringBuffer(())
        // RedisValue::SimpleString(())
        // RedisValue::SimpleString(())
    };

    // Обходим все ключи с помощью сканирования
    while cursor.scan(ctx, &scan_callback) {
        // Здесь нам ничего не надо делать пока что
    }

    Ok(RedisValue::Array(res))
}

//////////////////////////////////////////////////////

redis_module! {
    name: "scan",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [],
    init: init,
    deinit: deinit,
    // Регистрируемые команды редиса.
    // Формат: [имя, вызываемая функция, флаги, первый ключ, последний ключ, шаг]
    // Флаги можно посмотреть здесь поиском по `RedisModule_CreateCommand`: 
    // https://redis.io/docs/reference/modules/modules-api-ref/
    commands: [
        ["scan_keys", scan_keys, "readonly", 0, 0, 0],
    ],
}
