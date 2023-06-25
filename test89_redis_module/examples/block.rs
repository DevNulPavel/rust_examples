use redis_module::{
    redis_module, Context, RedisResult, RedisString, RedisValue, ThreadSafeContext,
};
use std::thread;
use std::time::Duration;

fn block(ctx: &Context, _args: Vec<RedisString>) -> RedisResult {
    // Создаем объект блокировки для конкретного
    // подключенного клиента.
    // TODO: но можно ли несколкьо раз создавать блокировку клиента?
    let blocked_client = ctx.block_client();

    // Запускаем уже какой-то наш фоновый поток исполнения
    thread::spawn(move || {
        // Создаем теперь в пределах потока исполнения
        // непосредственно тот же самый контекст,
        // который мы имеем при вызове обычно, но для потока текущего.
        let thread_ctx = ThreadSafeContext::with_blocked_client(blocked_client);

        {
            // Так же мы можем взять в фоновом потоке
            // исполнения блокировку основного потока редиса,
            // затем выполнить какие-то действия
            let lock_guard = thread_ctx.lock();

            // Можем вывести логи
            lock_guard.log_notice("Log from thread before");
        }

        // Подождем какое-то время
        thread::sleep(Duration::from_millis(5000));

        {
            // Так же мы можем взять в фоновом потоке
            // исполнения блокировку основного потока редиса,
            // затем выполнить какие-то действия
            let lock_guard = thread_ctx.lock();

            // Можем вывести логи
            lock_guard.log_notice("Log from thread after");
        }

        // С помощью контекста мы можем вернуть для клиента результат
        // определенный
        thread_ctx.reply(Ok("42".into()));
    });

    // Конкретно для такого запроса
    Ok(RedisValue::NoReply)
}

//////////////////////////////////////////////////////

redis_module! {
    name: "block",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    data_types: [],
    // Описание команды
    commands: [
        ["block", block, "", 0, 0, 0],
    ]
}
