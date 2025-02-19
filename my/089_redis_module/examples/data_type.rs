use redis_module::{
    native_types::RedisType, raw, redis_module, Context, NextArg, RedisModuleIO, RedisResult,
    RedisString,
};
use std::os::raw::c_void;

////////////////////////////////////////////////////////////////////////////////////////

/// Наша собственная структура с какими-то там данными.
#[derive(Debug)]
struct MyType {
    // В виде данных пока что у нас будет просто строка
    // data: String,

    // Можно так же использовать редис-строку, тогда должен вроде бы использоваться
    // общий аллокатор и тд.
    data: RedisString,
    // Можно так же использовать редис-строку, тогда должен вроде бы использоваться
    // общий аллокатор и тд.
    // data: RedisModuleString,
}

////////////////////////////////////////////////////////////////////////////////////////

/// Создаем статическую переменную, которая содержит описание нашего нового типа данных.
/// Подробнее можно почитать здесь по пункту `RedisModule_CreateDataType`:
/// - https://redis.io/docs/reference/modules/modules-api-ref/
/// - https://redis.io/docs/reference/modules/modules-native-types/
/// - https://redis.io/docs/reference/modules/modules-api-ref/#RedisModule_CreateDataType
static MY_REDIS_TYPE: RedisType = RedisType::new(
    // Уникальное имя нашего типа, состоящее из 9ти символов.
    // Это связано с тем, что внутри редиса данный ключ конвертируется в u64 значение при сохранении данных.
    // При ошибке, u64 снова конвертируется в строку и выдается подсказка при ошибке или отсутствии модуля.
    // Так же этот код выдается при вызове команды TYPE.
    //
    // Почему 9 символов, а не 8?
    // Возможно, что дело в том, что используется представление 127 символов ASCII, а не 255,
    // поэтому появляется запас для одного лишнего символа еще.
    //
    // Один символ - 6 бит. Остается при сохранении еще 10 бит на версию типа.
    // Иначе говоря: 6 * 9 + 10 = 64
    //
    // The type name is a 9 character name in the character
    // set that includes from A-Z, a-z, 0-9, plus
    // the underscore _ and minus - characters.
    "mytype123",
    // Версия нашего типа, нужно для возмиожности подгрузки из
    // бекапов данных о типах старшей версии.
    0,
    // Различные метода нашего нового типа для редиса.
    // Часть этих самых методов являются обязательными, а часть - нет.
    raw::RedisModuleTypeMethods {
        // Версия именно текущей библиотеки модулей, а не самих данных
        version: raw::REDISMODULE_TYPE_METHOD_VERSION as u64,
        // Функция загрузки данных из бекапа
        rdb_load: Some(load),
        // Функция сохранения данных в бекап
        rdb_save: Some(save),
        // Вызывается, когда AppenOnlyFile был перезаписан и надо его заполнить еще раз.
        aof_rewrite: None,
        // Функция по очистке данных в редисе, вызывается при удалении объекта по ключу
        free: Some(free),

        // Пока не используется редисом.
        // Вызывается при попытке узнать потребление данных объектом памяти во время вызова MEMORY
        mem_usage: None,

        // Цифровой хеш по вызову DEBUG DIGEST
        digest: None,

        // TODO: ???
        // Aux data
        aux_load: None,
        aux_save: None,
        aux_save_triggers: 0,

        free_effort: None,
        unlink: None,
        copy: None,
        defrag: None,

        copy2: None,
        free_effort2: None,
        mem_usage2: None,
        unlink2: None,
    },
);

////////////////////////////////////////////////////////////////////////////////////////

// Load
unsafe extern "C" fn load(rdb: *mut RedisModuleIO, encver: i32) -> *mut c_void {
    if encver == 0 {
        // Подгружаем чисто строчку содержимого из дампа
        match redis_module::load_string(rdb) {
            Ok(d) => {
                // TODO: Конвертация подсмотрена в `Context::set_value` методе ниже

                // Создаем сначала наш конкретный тип
                let v = MyType { data: d };

                // Теперь перемещаем его в кучу
                let boxed = Box::new(v);

                // Конвертируем в сырой указатель на данные и конвертируем тип указателя
                Box::into_raw(boxed).cast()
            }
            Err(_) => {
                // TODO: Что именно тут делать? Пока вернем просто нулевой указатель
                std::ptr::null_mut()
            }
        }
    } else {
        // TODO: Что именно тут делать? Пока вернем просто нулевой указатель
        std::ptr::null_mut()
    }
}

// Save
unsafe extern "C" fn save(rdb: *mut RedisModuleIO, value: *mut c_void) {
    // Сначала кастуем указатель на данные к указателю на наш тип
    let value_ptr = value.cast::<MyType>();

    // Теперь мы проверяем указатель на нулевой и пробразуем его в ссылку
    if let Some(value_ref) = value_ptr.as_ref() {
        // Если у нас есть объект, тогда просто записываем содержимое строки в дамп
        redis_module::save_redis_string(rdb, &value_ref.data);
    }
}

/// Данная сишная функция у нас вызывается при попытке очистить память нашей структурой
unsafe extern "C" fn free(value: *mut c_void) {
    // Другой пример деаллокации:
    // use std::alloc::{dealloc, Layout};
    // use std::ptr;
    // let x = Box::new(String::from("Hello"));
    // let p = Box::into_raw(x);
    // unsafe {
    //     ptr::drop_in_place(p);
    //     dealloc(p as *mut u8, Layout::new::<String>());
    // }

    // Сначала кастим сишный указатель к нашему конкретному типу
    // TODO: Учитывать ли порядок байтов litle/big endian ?
    let value_obj = value.cast::<MyType>();

    // Теперь преобразуем в box
    let boxed_value = Box::from_raw(value_obj);

    // Теперь явно уничтожаем память с помощью drop у box
    drop(boxed_value);
}

////////////////////////////////////////////////////////////////////////////////////////

/// Обработчик команды
fn alloc_set(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    // Получаем аргументы нашей команды, пропускаем самую первую, так как это сама команда
    let mut args = args.into_iter().skip(1);

    // Получаем параметр ключа
    let key = args.next_arg()?;

    // Получаем размер в виде цифры
    let size = args.next_i64()?;

    // Делаем отладочный вывод
    ctx.log_debug(format!("key: {key}, size: {size}").as_str());

    // Теперь по данному ключу мы открываем на запись буфер
    let key_write_guard = ctx.open_key_writable(&key);

    // Здесь мы можем получить попытаться мутабельную ссылку на нашу структуру
    // с помощью передачи в качестве параметра информации о типе.
    if let Some(value) = key_write_guard.get_value::<MyType>(&MY_REDIS_TYPE)? {
        // Если у нас такое значение уже было, заполняем его значениями B много раз
        value.data = ctx.create_string("B".repeat(size as usize));
    } else {
        // Если такого значения еще у нас не было, тогда мы создаем структурку
        // с нуля и заполняем значениями
        let value = MyType {
            data: ctx.create_string("A".repeat(size as usize)),
        };

        // Записываем это самое значение с помощью указания типа переменной
        key_write_guard.set_value(&MY_REDIS_TYPE, value)?;
    }
    Ok(size.into())
}

// Обработчик получения данных по ключу
fn alloc_get(ctx: &Context, args: Vec<RedisString>) -> RedisResult {
    // Получаем аргументы и пропускаем саму команду
    let mut args = args.into_iter().skip(1);

    // Получаем теперь сам ключ
    let key = args.next_arg()?;

    // Открываем ключ данный на чтение
    let key_read_guard = ctx.open_key(&key);

    // Пытаемся получить значение по данному ключу с помощью каста к типу + описания типа
    let value = match key_read_guard.get_value::<MyType>(&MY_REDIS_TYPE)? {
        Some(value) => {
            // Раз получили ссылку на данные, тогда можем извлечь и содержимое
            value.data.clone().into()
        }
        // Либо возвращаем пустоту
        None => ().into(),
    };

    Ok(value)
}

//////////////////////////////////////////////////////

redis_module! {
    name: "alloc",
    version: 1,
    allocator: (redis_module::alloc::RedisAlloc, redis_module::alloc::RedisAlloc),
    // Здесь мы описываем конкретный наш новый тип
    data_types: [
        MY_REDIS_TYPE,
    ],
    // Регистрируемые команды редиса.
    // Формат: [имя, вызываемая функция, флаги, первый ключ, последний ключ, шаг]
    // Флаги можно посмотреть здесь поиском по `RedisModule_CreateCommand`:
    // https://redis.io/docs/reference/modules/modules-api-ref/
    commands: [
        ["alloc.set", alloc_set, "write", 1, 1, 1],
        ["alloc.get", alloc_get, "readonly", 1, 1, 1],
    ],
}
