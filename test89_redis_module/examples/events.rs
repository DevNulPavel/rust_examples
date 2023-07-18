use redis_module::{
    redis_module, Context, NotifyEvent, RedisError, RedisResult, RedisString, RedisValue, Status,
};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicI64, Ordering};

////////////////////////////////////////////////////////////////////////////////////////

/// Статический глобальный атомарный счетчик ненайденных ключей
static NUM_KEY_MISSES: AtomicI64 = AtomicI64::new(0);

////////////////////////////////////////////////////////////////////////////////////////

/// Какое-то событие со строкой
fn on_string_event(ctx: &Context, event_type: NotifyEvent, event: &str, key: &[u8]) {
    // Если это рекурсивное действие со строкой, то прерываем вызов
    if key == b"num_sets" {
        // break infinit look
        return;
    }

    {
        // Создаем строчку для вывода в логи
        let msg = format!(
            "Received event: {:?} on key: {} via event: {}",
            event_type,
            std::str::from_utf8(key).unwrap(),
            event
        );

        // Пишем в лог редиса
        ctx.log_notice(msg.as_str());
    }

    // https://redis.io/docs/reference/modules/modules-api-ref/#RedisModule_AddPostNotificationJob
    // Внутри обработчика событий нежелательно запускать напрямую какие-то вызовы записи и тд
    // Данный коллбек будет выполнен тогда, кода будут условия:
    // - можем безопасно выполнять запись
    // - коллбек выполнится атомарно вместе с событием
    let _ = ctx.add_post_notification_job(|ctx| {
        // Теперь в контексте мы можем выполнить команду,
        // где мы просто увеличиваем счетчик по ключу на один
        if let Err(e) = ctx.call("incr", &["num_sets"]) {
            ctx.log_warning(&format!("Error on incr command, {}.", e));
        }
    });
}

////////////////////////////////////////////////////////////////////////////////////////

/// Какое-то там событие стрима
fn on_stream_event(ctx: &Context, _event_type: NotifyEvent, _event: &str, _key: &[u8]) {
    ctx.log_debug("Stream event received!");
}

////////////////////////////////////////////////////////////////////////////////////////

/// Обработчик отсутствия ключа
fn on_key_miss(_ctx: &Context, _event_type: NotifyEvent, _event: &str, _key: &[u8]) {
    // Здесь мы просто увеличиваем счетчик отсутствия ключей
    NUM_KEY_MISSES.fetch_add(1, Ordering::SeqCst);
}

////////////////////////////////////////////////////////////////////////////////////////

/// Обработчик команды отправки
fn event_send_command(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    // Аргумент должен быть один
    if args.len() > 1 {
        return Err(RedisError::WrongArity);
    }

    // Создаем строку в контексте
    let key_name = RedisString::create(NonNull::new(ctx.ctx), "mykey");

    // Создаем GENERIC событие с ключем
    let status = ctx.notify_keyspace_event(NotifyEvent::GENERIC, "events.send", &key_name);

    // Статус отправки события
    match status {
        Status::Ok => Ok("Event sent".into()),
        Status::Err => Err(RedisError::Str("Generic error")),
    }
}

/// Обработчик команды получения счетчика
fn num_key_miss_command(_ctx: &Context, _args: Vec<RedisString>) -> RedisResult {
    Ok(RedisValue::Integer(NUM_KEY_MISSES.load(Ordering::SeqCst)))
}

////////////////////////////////////////////////////////////////////////////////////////

redis_module! {
    name: "events",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [],
    // Регистрируемые команды редиса.
    // Формат: [имя, вызываемая функция, флаги, первый ключ, последний ключ, шаг]
    // Флаги можно посмотреть здесь поиском по `RedisModule_CreateCommand`:
    // https://redis.io/docs/reference/modules/modules-api-ref/
    commands: [
        ["events.send", event_send_command, "", 0, 0, 0],
        ["events.num_key_miss", num_key_miss_command, "", 0, 0, 0],
    ],
    // Обработчики различных событий в редисе:
    // - https://redis.io/docs/reference/modules/modules-api-ref/#RedisModule_SubscribeToKeyspaceEvents
    // - https://redis.io/docs/reference/modules/modules-api-ref/#RedisModule_GetNotifyKeyspaceEvents
    //
    // Регистрируем коллбеки на события, происходящие с определенными типами данных,
    // либо на конкретные действия.
    event_handlers: [
        // Обработчик событий над строками
        [@STRING: on_string_event],
        // Обработчик событий над стримом
        [@STREAM: on_stream_event],
        // Событие отсутствия нужного ключа в редисе?
        [@MISSED: on_key_miss],
    ],
}
