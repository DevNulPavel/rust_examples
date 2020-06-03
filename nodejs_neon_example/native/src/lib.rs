use neon::{
    prelude::{
        *
    }
};

fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    // Контекст нужен для работы со сборкой мусора, создания нативных переменных и тд
    // Handle - как раз управляет сборкой мусора в NodeJs
    let text: Handle<JsString> = cx.string("hello node");
    Ok(text)
}

fn number_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // Число должно обязательно каститься к f64
    Ok(cx.number(100 as f64))
}

/// Создаем массив и добавляем значения к нему
fn make_an_array(mut cx: FunctionContext) -> JsResult<JsArray> {
    // Создаем значения
    let num = cx.number(9000);
    let text = cx.string("hello");
    let bool_v = cx.boolean(true);
    let undef = cx.undefined();
    let null = cx.null();
    // Создаем наш массив
    let array: Handle<JsArray> = cx.empty_array();
    // Добавляем значения в массив
    array.set(&mut cx, 0, num)?;
    array.set(&mut cx, 1, text)?;
    array.set(&mut cx, 2, bool_v)?;
    array.set(&mut cx, 3, undef)?;
    array.set(&mut cx, 4, null)?;
    // Возвращаем
    Ok(array)
}

/// Создаем функцию, которая выдает количество аргументов, переданное ей
fn get_args_len(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let args_length = cx.len();
    let arg0: Handle<JsString> = cx.argument(0)?;
    let arg1: Handle<JsNumber> = cx.argument(1)?;
    let arg0_str: String = arg0.value();
    let arg1_f: f64 = arg1.value();
    println!("RUST: Args count = {}, [{}, {}]", args_length, arg0_str, arg1_f);
    Ok(cx.number(args_length))
}

fn test_objects(mut cx: FunctionContext) -> JsResult<JsObject>{
    // Создаем новый объект
    let js_object = JsObject::new(&mut cx);

    // Создаем функцию
    let func = JsFunction::new(&mut cx, |mut cx|{
        Ok(cx.string("Returned text from function"))
    })?;

    // Устанавливаем функцию
    js_object.set(&mut cx, "function_property", func)?;

    // Аналогичным образом получаем функцию
    let _func = js_object
        .get(&mut cx, "function_property")?
        .downcast::<JsFunction>() // Кастим к типу JsFunction
        .or_throw(&mut cx)?;      // Иначе выбрасываем Js исключение

    Ok(js_object)
}

// Asynchronously compute fibonacci on another thread
fn fibonacci_async(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    /*// Получаем аргумент 0
    let n = cx.argument::<JsNumber>(0)?.value() as usize;
    // Получаем аргумент 1 - коллбек
    let cb = cx.argument::<JsFunction>(1)?;
    
    let task = FibonacciTask{ 
        argument: n 
    };
    task.schedule(cb);*/
    
    Ok(cx.undefined())
}

// Экспортируем наши Rust функции
register_module!(mut cx, {
    cx.export_function("hello", hello)?;
    cx.export_function("number_function", number_function)?;
    cx.export_function("make_an_array", make_an_array)?;
    cx.export_function("get_args_len", get_args_len)?;
    cx.export_function("test_objects", test_objects)?;
    Ok(())
});