#![allow(dead_code)]

use std::ops::DerefMut;

////////////////////////////////////////////////////////////////////////////////////////////////////

// Описание интерфейса нашей библиотеки
#[link(name = "custom_lib", kind="static")]
extern {
    fn register_callback_int32(cb: extern "C" fn(i32)) -> i32;
    fn trigger_callback_int32();
    fn register_callback_obj(target: *mut RustObject, cb: extern "C" fn(*mut RustObject, i32)) -> i32;
    fn trigger_callback_obj();
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Функция, которую надо вызывать из C - помечается как extern, нужно для линковки
extern "C" fn callback_int(a: i32) {
    println!("Меня вызывают из C со значением {0}", a);
}

fn test_simple_cb(){
    unsafe {
        register_callback_int32(callback_int);
        trigger_callback_int32(); // Активация функции обратного вызова
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////

// Для таких структур обязательно нужно указывать представление в виде C
#[repr(C)]
struct RustObject {
    a: i32,
    // другие поля
}

// Функция, которую надо вызывать из C - помечается как extern, нужно для линковки
extern "C" fn callback_obj(target: *mut RustObject, a: i32) {
    println!("Меня вызывают из C со значением {0}", a);
    unsafe {
        // Меняем значение в RustObject на значение, полученное через функцию обратного вызова
        (*target).a = a;
    }
}


fn test_obj_cb() {
    // Создаём объект в куче с помозью Box, на который будем ссылаться в функции обратного вызова
    let mut rust_object: Box<RustObject> = Box::new(RustObject { 
        a: 0
    });

    unsafe {
        // Получаем указатель на наш сырой объект в куче
        // let raw_pointer: *mut RustObject = &mut *rust_object; - то же самое, что ниже
        let raw_pointer: &mut RustObject = rust_object.deref_mut();
        register_callback_obj(raw_pointer, callback_obj);
        trigger_callback_obj();
    }

    println!("{}", rust_object.a);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn test_custom_lib() {
    test_simple_cb();
    test_obj_cb();
}