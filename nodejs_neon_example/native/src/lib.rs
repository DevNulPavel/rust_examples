mod simple;
mod struct_example;
mod async_example;

use neon::{
    prelude::{
        *
    },
    //borrow::{
        //Ref,
        //RefMut
    //}
};
use crate::{
    struct_example::{
        //Employee,
        JsEmployee
    },
    simple::{
        *
    },
    async_example::{
        perform_async_task
    }
};


// Экспортируем наши Rust функции
register_module!(mut cx, {
    cx.export_function("hello", hello)?;
    cx.export_function("number_function", number_function)?;
    cx.export_function("make_an_array", make_an_array)?;
    cx.export_function("get_args_len", get_args_len)?;
    cx.export_function("test_objects", test_objects)?;
    cx.export_function("modify_object_this", modify_object_this)?;
    cx.export_function("function_as_parameter", function_as_parameter)?;
    cx.export_function("construct_js_function", construct_js_function)?;
    cx.export_function("perform_async_task", perform_async_task)?;
    
    // JsEmployee враппер говорит neon, какой именно класс мы экспортируем
    // "Employee" - это имя класса в JS
    cx.export_class::<JsEmployee>("Employee")?;

    Ok(())
});