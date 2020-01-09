#![allow(dead_code)]

extern crate libc;

use std::ops::DerefMut;
use std::ffi::CString;
use std::ffi::CStr;
use std::os::raw::c_char;

////////////////////////////////////////////////////////////////////////////////////////////////////

// Описание интерфейса нашей библиотеки
#[link(name = "custom_lib", kind="static")]
extern {
    fn register_callback_int32(cb: extern "C" fn(i32)) -> i32;
    fn trigger_callback_int32();
    fn register_callback_obj(target: *mut RustObject, cb: extern "C" fn(*mut RustObject, i32)) -> i32;
    fn trigger_callback_obj();
    fn test_string_code(text: *const c_char) -> *const c_char;
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

    // Получаем указатель на наш сырой объект в куче, что интересно - это все вне unsafe блока
    // Данная конструкция ниже читается так изнутри - наружу
    //      - сначала мы разыменовываем сырой указатель для полу чения нашего объекта в памяти
    //      - затем мы берем ссылку на тот самый объект в памяти
    //      - приводим эту ссылку к указателю, срасывая отслеживание lifetime
    let raw_pointer: *mut RustObject = &mut *rust_object; // то же самое, что ниже
    // let raw_pointer: &mut RustObject = rust_object.deref_mut();

    unsafe {
        register_callback_obj(raw_pointer, callback_obj);
        trigger_callback_obj();
    }

    println!("{}", rust_object.a);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn test_string_to_c(){
    // Создаем строку с NULL в конце
    let to_c_string = CString::new("Hello, world!").expect("CString::new failed");

    let c_str = unsafe {
        // Вызываем C-шную функцию с данными
        let text = test_string_code(to_c_string.as_ptr());
        // Оборачиваем указатель на данные в Rust ссылку
        CStr::from_ptr(text).to_owned()
    };
    // Создаем CopyOnWrite объект, который владеет данными, но копируем из только при необходимости
    //let from_c_string = c_str.to_string_lossy();

    // Создаем строку RUST
    let from_c_string = c_str.to_str().unwrap();

    println!("{}", from_c_string);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

fn test_some_unsafe_code(){
    fn get_pair_mut_from_array<T>(arr: &mut [T], index_1: usize, index_2: usize)-> (&mut T, &mut T){
        // Здесь мы приводим ссылку к сырому указателю + узбавляемся от lifetime нашей мутабельной ссылки
        // Что интересно - мы это делаем не в unsafe блоке
        let var_1: *mut T = &mut (arr[index_1]);
        let var_2: *mut T = &mut (arr[index_2]);

        unsafe{
            // Assert работает даже в release сборке, это значит, что будет происходить проверка всегда в коде
            assert_ne!(index_1, index_2);
            // Данная конструкция ниже читается так изнутри - наружу
            //      - сначала мы разыменовываем сырой указатель для получения нашего объекта в памяти
            //      - затем мы берем ссылку на тот самый объект в памяти
            (&mut *var_1, &mut *var_2)
        }
    }
    
    let mut test_array = [10, 20, 30, 40];
    //  Вызов с одинаковыми параметрами приведет к UndefinedBehaviour, для этого внутри assert
    let pair = get_pair_mut_from_array(&mut test_array, 0, 1);
    println!("{:?}", pair);
}

////////////////////////////////////////////////////////////////////////////////////////////////////

pub fn test_custom_lib() {
    test_simple_cb();
    test_obj_cb();
    test_string_to_c();
    test_some_unsafe_code();
}