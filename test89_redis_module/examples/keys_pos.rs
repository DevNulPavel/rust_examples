use redis_module::{redis_module, Context, RedisError, RedisResult, RedisString, RedisValue};

fn keys_pos(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    // Количество аргументов без имени команды должно быть четным.
    if (args.len() - 1) % 2 != 0 {
        return Err(RedisError::WrongArity);
    }

    // Return non-zero if a module command, that was declared with the flag "getkeys-api",
    // is called in a special way to get the keys positions
    // and not to get executed. Otherwise zero is returned.
    //
    // Возвращает true если команда модуля, которая была объявлена с флагом `getkeys-api`,
    // была вызвана специальным способом для получения позиций ключей и не выполняется.
    // Иначе возвращается false.
    //
    // Используется синтаксис команд вида: start/stop/step 
    if ctx.is_keys_position_request() {
        // Перебираем переданные аргументы кроме имени самой команды
        for i in 1..args.len() {
            // У нас через один элемент идет номер команды и сама команда?
            if (i - 1) % 2 == 0 {
                ctx.key_at_pos(i as i32);
            }
        }
        Ok(RedisValue::NoReply)
    } else {
        // Если поддержки нету, тогда мы просто отбрасываем имя самой команды.
        // После этого мы итерируемся через каждый 2й элмент: 0, 2, 4
        let reply: Vec<_> = args.iter().skip(1).step_by(2).collect();

        // Возвращаем результат
        Ok(reply.into())
    }
}

//////////////////////////////////////////////////////

redis_module! {
    name: "keys_pos",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [],
    // Регистрируемые команды редиса.
    // Формат: [имя, вызываемая функция, флаги, первый ключ, последний ключ, шаг]
    // Флаги можно посмотреть здесь поиском по `RedisModule_CreateCommand`:
    // https://redis.io/docs/reference/modules/modules-api-ref/
    commands: [
        ["keys_pos", keys_pos, "getkeys-api", 1, 1, 1],
    ],
}
