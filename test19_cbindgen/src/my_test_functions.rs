// #![feature(test)]
// #![allow(soft_unstable)]
#![allow(unused_variables)]
#![allow(unused_macros)]
#![allow(unused_imports)]

// #[macro_use]
// extern crate lazy_static;
// extern crate libc;

// extern crate test;

use std::ffi::CStr;
use std::os::raw::c_char;
use std::sync::Mutex;
use std::collections::HashMap;
use std::thread::ThreadId;
use std::cell::RefCell;
use std::cell::Cell;
use std::borrow::Cow;
// use std::sync::Arc;
// use std::rc::Rc;
use lazy_static::lazy_static;


// Перед структурой можно указывать кастомные параметры
/// cbindgen:field-names=[data, len]
/// cbindgen:derive-eq

#[derive(Debug)]
#[repr(C)]
pub struct Buffer<T> {
    data_array: [T; 8],
    len: usize,
}

// https://doc.rust-lang.org/nomicon/ffi.html
// Просмотр списка функций в библиотеке - nm target/debug/libtest19_cbindgen.a | rg "func"

// Выстовляем соглашение о вызовах C + отключаем изменение имени функции
#[no_mangle]
pub extern "C" fn function_1(param: i32) -> i32 {
    println!("Value: {}", param);
    param
}

// Выставляем соглашение о вызовах C + отключаем изменение имени функции
#[no_mangle] 
pub extern "C" fn function_2(buffer: Buffer<i32>) -> i32 {
    println!("{:?}", buffer);
    let mut result = 0 as i32;
    for index in 0..buffer.len{
        if let Some(val) = buffer.data_array.get(index) {
            result += val;
        }else{
            break;
        }
    }
    result
}

// Выставляем соглашение о вызовах C + отключаем изменение имени функции
#[no_mangle] 
pub extern "C" fn test_raw_pointers() {
    // Способы получения указателя
    {
        // Можно получить указатель из ссылки, при этом - это безопасная операция, блок unsafe не нужен
        let my_num: i32 = 10;
        let my_num_ptr: *const i32 = &my_num;
        // Можно получить мутабельный указатель из ссылки
        let mut my_speed: i32 = 88;
        let my_speed_ptr: *mut i32 = &mut my_speed;
    }

    // Способ получения невладеющего указателя из умного указателя, но работать с ним можно только пока существует объект
    {
        // let mut test_ptr: *mut i32 = std::ptr::null_mut();

        // Получаем обычный указатель с помощью разыменования и получения ссылки
        let my_num: Box<i32> = Box::new(10);
        let my_num_ptr: *const i32 = &*my_num;

        // Точно так же можно получить указатель на мутабельные данные
        let mut my_speed: Box<i32> = Box::new(88);
        let my_speed_ptr: *mut i32 = &mut *my_speed;
        unsafe{
            *my_speed_ptr = 99;
        }
        println!("Box val: {}", *my_speed);

        // Такое можно сделать, но это неправильно
        // test_ptr = &mut *my_speed;

        // Уничтожаем умный указатель
        drop(my_speed);

        // if test_ptr.is_null() == false {
        //     unsafe{
        //         *test_ptr = 99;
        //     }
        // }
    }

    // Как вариант, можно теперь умный указатель в сырой указатель, затем обратно
    {
        let my_speed: Box<i32> = Box::new(88);
        // Владение перейдет на сырой указатель из обычного, данные не уничтожатся
        let my_speed: *mut i32 = Box::into_raw(my_speed);

        // By taking ownership of the original `Box<T>` though
        // we are obligated to put it together later to be destroyed.
        let source_box = unsafe {
            Box::from_raw(my_speed)
        };

        // Уничтожаем объект, все ок
        drop(source_box);
    }

    // Помимо прочего, можно аллоцировать все руками и уничтожать руками
    /*{
        unsafe {
            let int32_size = std::mem::size_of::<i32>();
            // Можно вызывать метод cast у указателя для приведения типов вместо оператора as
            let my_num: *mut i32 = libc::malloc(int32_size).cast::<i32>(); //  as *mut i32
            if my_num.is_null() {
                panic!("failed to allocate memory");
            }
            libc::free(my_num.cast::<libc::c_void>()); // my_num as *mut libc::c_void
        }
        println!("Manual alloc/dealloc success");
    }*/

    {
        let s: [i32; 7] = [1, 2, 3, 4, 5, 6, 7];
        let ptr: *const i32 = s[..3].as_ptr();
        // Можно смещать указатель на нужное значение элементов
        let ptr = unsafe { ptr.offset(1) };
        let ptr = unsafe { ptr.add(1) };
        // Wrap вызовы значат, что внутри уже есть вызов unsafe
        // Еще одно отличие в том, что при выходе за границы исходного массива wrap метод выдаст нормальный указатель, пусть и опасный для разыменования
        // В то время как обычный указатель - сразу приведет к неопределенному поведению
        let ptr = ptr.wrapping_add(1);
        assert!(!ptr.is_null());

        // Можно прочитать значение у данного указателя с созданием копии объекта? не изменяя исходную память
        let value_at_pos = unsafe{ ptr.read() };
        println!("Value at position: {}", value_at_pos);

        // Есть специальный метод as_ref, который превращает обычный указатель в Option со ссылкой
        // Если объект есть - значит все ок, указатель был не null
        let source_ref_option = unsafe { ptr.as_ref() };
        if let Some(source) = source_ref_option {
            println!("Source slice: {}", source);
        }
    }
}

#[no_mangle] 
pub extern "C" fn test_panic_catch() {
    // https://doc.rust-lang.org/std/panic/fn.catch_unwind.html

    // Мы можем запускать что-то внутри блока, чтобы отлавливать ошибки
    let result = std::panic::catch_unwind(|| {
        println!("hello!");
    });
    assert!(result.is_ok());

    // Отлавливаем ошибку внутри блока, но нужно помнить, что не все ошибки могут отлавливаться
    let result = std::panic::catch_unwind(|| {
        panic!("oh no!");
    });
    assert!(result.is_err());

    // Однако имеется возможность вызвать панику с сохранением стека и тд
    // как-то можно использовать для проброса паники из C
    /*{
        let result = std::panic::catch_unwind(|| {
            panic!("oh no!");
        });
        if let Err(err) = result {
            std::panic::resume_unwind(err);
        }    
    }*/

    // Так же мы можем назначить обработчик, который вызывается перед системным вызовом паники
    {
        std::panic::set_hook(Box::new(|panic_info| {
            //panic_info: std::panic::PanicInfo
    
            // Можно получить метаинформацию о краше в виде строки
            if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
                println!("panic occurred: {:?}", s);
            } else {
                println!("panic occurred");
            }
    
            // Можно пполучить месторасположение
            if let Some(location) = panic_info.location() {
                println!("panic occurred in file '{}' at line {}, column {}", 
                    location.file(), location.line(), location.column());
            } else {
                println!("panic occurred but can't get location information...");
            }
        }));
        panic!("Normal panic");
    }
    
}

/*#[no_mangle] 
pub extern "C" fn icmp1_RUST_CODE(s2: *const c_char, s1: *const c_char) -> i32 {
    if s2.is_null(){
        return 0;
    }

    // Создаем Rust ссылочную строку из С-шной
    let s2_convert_result = unsafe { CStr::from_ptr(s2).to_str() };

    // Если не смогли сконвертить в Rust строку - выходим
    let s2_test_str = match s2_convert_result {
        Ok(string) =>{
            string
        },
        Err(_) => {
            return 0;
        }
    };

    lazy_static! {
        static ref THREAD_STORRAGES: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));
    }

    // Если есть исходная строка, тогда делаем ее в нижнем регистре
    if s1.is_null() == false {
        // Создаем Rust ссылочную строку из С-шной
        let s1_convert_result = unsafe { CStr::from_ptr(s1).to_str() };

        // Если не смогли сконвертить в Rust строку - выходим
        if s1_convert_result.is_err() {
            return 0;
        }

        // Переводим в нижний регистр
        let s1_lowercase = s1_convert_result.unwrap().to_ascii_lowercase();
        
        match THREAD_STORRAGES.lock(){
            Ok(mut storrage) => {
                storrage.clone_from(&s1_lowercase);
            },
            Err(_)=>{
                return 0;
            }
        }
    }
    
    match THREAD_STORRAGES.lock(){
        Ok(storrage) => {
            let equals = s2_test_str.eq(&(*storrage));
            if equals {
                return 1;
            }else{
                return 0;
            }
        },
        Err(_)=>{
            return 0;
        }
    }
}*/

/*#[no_mangle] 
pub extern "C" fn icmp1_RUST_CODE(s2: *const c_char, s1: *const c_char) -> i32 {
    if s2.is_null(){
        return 0;
    }

    // Создаем Rust ссылочную строку из С-шной
    let s2_convert_result = unsafe { CStr::from_ptr(s2).to_str() };

    // Если не смогли сконвертить в Rust строку - выходим
    let s2_test_str = match s2_convert_result {
        Ok(string) =>{
            string
        },
        Err(_) => {
            return 0;
        }
    };

    /*lazy_static! {
        static ref THREAD_STORRAGES: Mutex<HashMap<ThreadId, Arc<Mutex<String>>>> = Mutex::new(HashMap::new());
    }

    let storrage = match THREAD_STORRAGES.lock(){
        Ok(mut locked_storrage) => {
            let cur_thread_id = std::thread::current().id();
        
            let storrage = locked_storrage.entry(cur_thread_id).or_insert(Arc::new(Mutex::new(String::new())));

            storrage.clone()
        },
        Err(_) => {
            return 0;
        }
    };

    // Если есть исходная строка, тогда делаем ее в нижнем регистре
    let equals = if s1.is_null() == false {
        // Создаем Rust ссылочную строку из С-шной
        let s1_convert_result = unsafe { CStr::from_ptr(s1).to_str() };

        // Если не смогли сконвертить в Rust строку - выходим
        if s1_convert_result.is_err() {
            return 0;
        }

        // Переводим в нижний регистр
        let s1_lowercase = s1_convert_result.unwrap().to_ascii_lowercase();

        match storrage.lock() {
            Ok(mut data)=>{
                (*data).clone_from(&s1_lowercase);

                let equals = s2_test_str.eq(&(*data));
                equals
            },
            Err(_)=>{
                return 0;
            }
        }
    }else{
        match storrage.lock() {
            Ok(data)=>{
                let equals = s2_test_str.eq(&(*data));
                equals
            },
            Err(_)=>{
                return 0;
            }
        }
    };

    if equals {
        return 1;
    }else{
        return 0;
    }*/

    // Если есть исходная строка, тогда делаем ее в нижнем регистре
    let s1_lowercase = if s1.is_null() == false {
        // Создаем Rust ссылочную строку из С-шной
        let s1_convert_result = unsafe { CStr::from_ptr(s1).to_str() };

        // Если не смогли сконвертить в Rust строку - выходим
        if s1_convert_result.is_err() {
            return 0;
        }

        // Переводим в нижний регистр
        let s1_lowercase = s1_convert_result.unwrap().to_ascii_lowercase();

        Some(s1_lowercase)
    }else{
        None
    };

    lazy_static! {
        static ref THREAD_STORRAGES: Mutex<HashMap<ThreadId, String>> = Mutex::new(HashMap::new());
    }

    let cur_thread_id = std::thread::current().id();

    // Пользуемся блокировкой для обновления статической переменной
    match THREAD_STORRAGES.lock(){
        Ok(mut locked_storrage) => {
                   
            let storrage = locked_storrage.entry(cur_thread_id).or_insert(String::new());

            if let Some(lowercase_str) = s1_lowercase{
                *storrage = lowercase_str;
            }

            if s2_test_str.eq(storrage) {
                return 1;
            }else{
                return 0;
            }
        },
        Err(_) => {
            return 0;
        }
    };
}*/

#[no_mangle] 
pub extern "C" fn icmp1_RUST_CODE(s2: *const c_char, s1: *const c_char) -> i32 {
    if s2.is_null() {
        return 0;
    }

    // Создаем Rust ссылочную строку из С-шной
    // Если не смогли сконвертить в Rust строку - выходим
    let s2_test_str = match unsafe { CStr::from_ptr(s2).to_str() } {
        Ok(string) =>{
            string
        },
        Err(_) => {
            return 0;
        }
    };

    // Защита от кривых данных
    if s2_test_str.len() > 1024{
        return 0;
    }

    struct Storrage{
        buffer: Vec<u8>,
        string_len: usize,
        // str_raw: &'a str
        //str_raw: RefCell<Cow<'a, str>>,
        // str_raw: Cow<'a, str>,
    };
    impl Storrage{
        fn new() -> Storrage {
            // let buffer: Vec<u8> = Vec::new();
            
            // let bytes_slice: &[u8] = ;
            // let string = String::from_utf8_lossy(&buffer[0..0]);
            // let test_str: &str = ;

            let st = Storrage{
                buffer: Vec::new(),
                string_len: 0,
                // str_raw: RefCell::new(String::from_utf8_lossy(&buffer[0..0]))
                // str_raw: RefCell::new(Cow::default())
                // str_raw: Cow::default()
                // str_raw: ""
            };
            // st.str_raw = RefCell::new(String::from_utf8_lossy(&st.buffer[0..0]));

            st
        }
    }

    thread_local!{
        static THREAD_STORRAGES: RefCell<Storrage> = RefCell::new(Storrage::new());
    }

    // Если есть исходная строка, тогда делаем ее в нижнем регистре
    if s1.is_null() == false {
        // Создаем Rust ссылочную строку из С-шной
        // Если не смогли сконвертить в Rust строку - выходим
        match unsafe { CStr::from_ptr(s1).to_str() } {
            Ok(res) =>{
                // Защита от кривых данных
                if res.len() > 1024{
                    return 0;
                }

                THREAD_STORRAGES.with(|st| {
                    // RefCell::new(String::new()).get_mut();
                    let storrage = &mut *st.borrow_mut();

                    // Увеличиваем размер буффера
                    if storrage.buffer.len() < res.len(){
                        // println!("Resize {}", res.len());
                        storrage.buffer.resize(res.len(), 0);
                    }

                    // let mut bytes_iter = String::new().chars();
                    //bytes_iter = 2 as char;
                    // *(bytes_iter.by_ref()) = 2 as u8;
                    // bytes_iter.next();

                    // Переводим в нижний регистр
                    let mut i = 0 as usize;
                    // let mut src_string_it = st.borrow_mut().chars();
                    // let vec = unsafe { src_string.as_mut_vec() };
                    let bytes_iter = res.chars();
                    for byte in bytes_iter {
                        let lowercase_byte = byte.to_ascii_lowercase() as u8;
                        if let Some(val) = storrage.buffer.get_mut(i) {
                            *val = lowercase_byte;
                        }

                        // src_string.insert(i, lowercase_byte);

                        // String::new().
                        // let vec: Vec<u8> = Vec::new();
                        // vec.get

                        // src_string_it = lowercase_byte;
                        // src_string_it.next();

                        // if let Some(val) = src_string.get_mut(i) {
                        //     *val = lowercase_byte;
                        // }
                        i += 1;
                    }
                    storrage.string_len = i;

                    // let bytes_slice: &[u8] = &storrage.buffer[..storrage.string_len];
                    // let string = String::from_utf8_lossy(bytes_slice);
                    // let test_str: &str = &*string;
                    // storrage.str_raw = RefCell::new(string);
                    // storrage.str_raw = string;
                    // storrage.str_raw = test_str;

                    // String::from_utf8(bytes).unwrap()
                    // Cell::new(String::new()).set()
                    // ;

                    // src_string.truncate(i);

                    // println!("TEST {}", src_string);
                });
            }
            Err(_)=>{
                return 0;
            }
        }
    }

    return THREAD_STORRAGES.with(|st| {
        //(*(RefCell::new(Storrage::new()).borrow()))
        let storrage: &Storrage = &(*st.borrow());

        let bytes_slice: &[u8] = &storrage.buffer[..storrage.string_len];
        let string = String::from_utf8_lossy(bytes_slice);
        let test_str: &str = &*string;

        if s2_test_str.eq(&(*test_str)) {
            return 1;
        }else{
            return 0;
        }
    });
}


macro_rules! to_c_str {
    ($val:expr) => (
        std::ffi::CString::new($val).unwrap().as_ptr()
    )
}

macro_rules! test_func {
    // У макроса может быть несколько вариантов
    ($val1:expr, $val2:expr) => (
        icmp1_RUST_CODE(to_c_str!($val1), to_c_str!($val2))
    );
    ($val1:expr) => (
        icmp1_RUST_CODE(to_c_str!($val1), std::ptr::null())
    )
}

pub fn test_icmp(){
    assert_eq!(test_func!("test1", "test1"), 1);
    assert_eq!(test_func!("test1", "TEST2"), 0);
    assert_eq!(test_func!("test2"), 1);
    assert_eq!(test_func!("test0"), 0);
    assert_eq!(test_func!(""), 0);
    assert_eq!(test_func!("  фывфыв"), 0);
    assert_eq!(test_func!("asdasd", "ASDASD"), 1);
    assert_eq!(test_func!("asda", "ASDASD"), 0);
    assert_eq!(test_func!("as", "ASD"), 0);
    assert_eq!(test_func!("as", "ASDDADASDS"), 0);
    assert_eq!(test_func!("asddadasds", "ASDDADASDS"), 1);
    assert_eq!(test_func!("add", "ASDDADASDS"), 0);
    assert_eq!(test_func!("add"), 0);
    assert_eq!(test_func!("asddadasds"), 1);
    assert_eq!(test_func!("asd", "ASD"), 1);
    assert_eq!(test_func!("asd"), 1);
    assert_eq!(test_func!("asd_"), 0);
    assert_eq!(test_func!("asd____"), 0);
    assert_eq!(test_func!("a"), 0);
    assert_eq!(test_func!(""), 0);
}

#[cfg(test)]
mod tests {
    use super::*;
    //use crate::my_test_functions::*;
    // use test::Bencher;

    #[test]
    fn icmp1_test() {
        test_icmp();
    }

    // #[bench]
    // fn icmp1_bench(b: &mut Bencher) {
    //     b.iter(|| {
    //         test_func!("test1", "test1");
    //         test_func!("test1", "TEST2");
    //         test_func!("test2");
    //         test_func!("test0");
    //         test_func!("");
    //         test_func!("  фывфыв");
    //         test_func!("asdasd", "ASDASD");
    //         test_func!("asda", "ASDASD");
    //         test_func!("as", "ASD");
    //         test_func!("as", "ASDDADASDS");
    //         test_func!("asddadasds", "ASDDADASDS");
    //         test_func!("add", "ASDDADASDS");
    //         test_func!("add");
    //         test_func!("asddadasds");
    //         test_func!("asd", "ASD");
    //         test_func!("asd");
    //         test_func!("asd_");
    //         test_func!("asd____");
    //         test_func!("a");
    //         test_func!("");
    //     });
    // }
}
