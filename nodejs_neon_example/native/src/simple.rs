use neon::{
    prelude::{
        *
    },
    //borrow::{
        //Ref,
        //RefMut
    //}
};

pub fn hello(mut cx: FunctionContext) -> JsResult<JsString> {
    // Контекст нужен для работы со сборкой мусора, создания нативных переменных и тд
    // Handle - как раз управляет сборкой мусора в NodeJs
    let text: Handle<JsString> = cx.string("hello node");
    Ok(text)
}

pub fn number_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // Число должно обязательно каститься к f64
    Ok(cx.number(100 as f64))
}

/// Создаем массив и добавляем значения к нему
pub fn make_an_array(mut cx: FunctionContext) -> JsResult<JsArray> {
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
pub fn get_args_len(mut cx: FunctionContext) -> JsResult<JsNumber> {
    let args_length = cx.len();
    let arg0: Handle<JsString> = cx.argument(0)?;
    let arg1: Handle<JsNumber> = cx.argument(1)?;
    let arg0_str: String = arg0.value();
    let arg1_f: f64 = arg1.value();
    println!("RUST: Args count = {}, [{}, {}]", args_length, arg0_str, arg1_f);
    Ok(cx.number(args_length))
}

pub fn test_objects(mut cx: FunctionContext) -> JsResult<JsObject>{
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

pub fn modify_object_this(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Получаем наш объект this, контекст функции
    // Обычные функции имеют свой контекст, лямбды захватывают контекст this вызвавшей функции
    let this = cx.this();
    
    // Приводим this к типу JsObject, чтобы можно было вызвать .set
    let this = this.downcast::<JsObject>().or_throw(&mut cx)?;

    // Создаем переменную bool
    let t = cx.boolean(true);
    
    // Устанавливаем значение переменной modified у объекта, аналогично "this.modified = true"
    this.set(&mut cx, "modified", t)?;

    Ok(cx.undefined())
}

pub fn function_as_parameter(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Получаем нашу переданную функцию
    let func = cx.argument::<JsFunction>(0)?;

    // Создаем массив аргуметов нашей функции
    /*let args: [Handle<JsNumber>; 1] = [cx.number(16.0)];
    let iter = args
        .into_iter()
        .map(|val| { 
            &val 
        });*/

    // Создаем владеющий вектор аргуметов нашей функции
    let args: Vec<Handle<JsNumber>> = vec![cx.number(16.0)];
    
    // Контекст this будет null
    let null_this = cx.null();

    // Вызываем нашу JS функцию с параметрами
    let received_number = func
        .call(&mut cx, null_this, args)?
        .downcast::<JsNumber>()
        .or_throw(&mut cx)?
        .value();
    
    println!("RUST: value {}", received_number);

    Ok(cx.undefined())
}

// TODO: Test it
/// Мы можем создать JS функцию с контекстом
pub fn construct_js_function(mut cx: FunctionContext) -> JsResult<JsNumber> {
    // Получаем функцию как параметр, передаем функцию Date, которая ведет себя как класс
    let param_js_func = cx.argument::<JsFunction>(0)?;

    // Нулевое значение
    //let zero = cx.number(0.0);
    let text = cx.string("December 31, 1975, 23:15:30 GMT+11:00");

    // Создаем новую функцию, но уже с замыканием определенных значений?
    // Аналогично: const date = new Date('December 31, 1975, 23:15:30 GMT+11:00');
    let date_obj = param_js_func.construct(&mut cx, vec![text])?;

    // Получаем метод getUTCFullYear у нового объекта-функции
    let get_utc_full_year_method = date_obj
        .get(&mut cx, "getUTCFullYear")? // Получаем функцию getUTCFullYear у объекта
        .downcast::<JsFunction>()
        .or_throw(&mut cx)?;

    // Аргументы
    let args: Vec<Handle<JsValue>> = vec![];
    let out_js_val = date_obj.upcast::<JsValue>();

    // Вызываем данный метод
    get_utc_full_year_method
        .call(&mut cx, out_js_val, args)?
        .downcast::<JsNumber>()
        .or_throw(&mut cx)
}


// Asynchronously compute fibonacci on another thread
/*fn fibonacci_async(mut cx: FunctionContext) -> JsResult<JsUndefined> {
    // Получаем аргумент 0
    let n = cx.argument::<JsNumber>(0)?.value() as usize;
    // Получаем аргумент 1 - коллбек
    let cb = cx.argument::<JsFunction>(1)?;
    
    let task = FibonacciTask{ 
        argument: n 
    };
    task.schedule(cb);
    
    Ok(cx.undefined())
}*/