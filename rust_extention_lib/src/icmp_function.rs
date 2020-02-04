
use std::ffi::CStr;
use std::os::raw::c_char;
use std::cell::RefCell;

// https://doc.rust-lang.org/nomicon/ffi.html

#[no_mangle] 
pub extern "C" fn icmp1_RUST_CODE(s2: *const c_char, s1: *const c_char) -> i32 {
    if s2.is_null() {
        return 0;
    }

    // Создаем Rust ссылочную строку из С-шной
    // Если не смогли сконвертить в Rust строку - выходим
    let cstr = unsafe { CStr::from_ptr(s2) };
    let s2_test_str = cstr.to_bytes();

    // Защита от кривых данных
    if s2_test_str.len() > 1024{
        return 0;
    }

    struct Storrage{
        buffer: Vec<u8>,
        string_len: usize,
    };
    impl Storrage{
        fn new() -> Storrage {
            Storrage {
                buffer: Vec::new(),
                string_len: 0,
            }
        }
    }

    thread_local!{
        static THREAD_STORRAGES: RefCell<Storrage> = RefCell::new(Storrage::new());
    }

    // Если есть исходная строка, тогда делаем ее в нижнем регистре
    if s1.is_null() == false {
        // Создаем Rust ссылочную строку из С-шной
        // Если не смогли сконвертить в Rust строку - выходим
        let cstr = unsafe { CStr::from_ptr(s1) };
        let res = cstr.to_bytes();

        // Защита от кривых данных
        if res.len() > 1024{
            return 0;
        }

        return THREAD_STORRAGES.with(|st| {
            let storrage: &mut Storrage = &mut *st.borrow_mut();

            // Увеличиваем размер буффера
            if storrage.buffer.len() < res.len(){
                storrage.buffer.resize(res.len(), 0);
            }

            // Переводим в нижний регистр
            let length = res.len();
            // Тут проверки выхода за границы происходят каждый раз внутри цикла
            /*for i in 0..length {
                let byte = res[i];
                let lowercase_byte = byte.to_ascii_lowercase();
                storrage.buffer[i] = lowercase_byte;
            }*/
            /*let mut i = 0;
            for &byte in res[0..length].iter() {
                let lowercase_byte = byte.to_ascii_lowercase();
                storrage.buffer[i] = lowercase_byte;
                i += 1;
            }*/
            // Проверки индексов происходят только при создании слайса, в самом цикле не происходит никаких проверок
            // В теории - самый высокопроизводительный вариант
            let iter = res
                .into_iter() // Перемещает владение содержимого
                //.iter()    // Работает со ссылками
                .zip(storrage.buffer[0..length].iter_mut());
            for byte_pair in iter {
                let lowercase_byte = byte_pair.0.to_ascii_lowercase();
                *(byte_pair.1) = lowercase_byte;
            }
            /*res[0..length]
                .iter()
                .zip(storrage.buffer[0..length].iter_mut())
                .for_each(|byte_pair|{
                    let lowercase_byte = byte_pair.0.to_ascii_lowercase();
                    *(byte_pair.1) = lowercase_byte;
                });*/
            storrage.string_len = length;

            // Вариант без проверок границы
            unsafe{
                let bytes_slice: &[u8] = storrage.buffer.get_unchecked(0..length);
                if s2_test_str.eq(bytes_slice) {
                    return 1;
                }
            }
            // Вызывается метод index из трейта Index, который возвращает ссылку и паникует при выходе за границы
            /*let ref bytes_slice = storrage.buffer[0..length];
            if s2_test_str.eq(bytes_slice) {
                return 1;
            }*/  
            // Способ ниже с проверкой выхода за границы, get выдает ссылку на слайс
            /*if let Some(bytes_slice) = storrage.buffer.get(0..length){
                if s2_test_str.eq(bytes_slice) {
                    return 1;
                }   
            }*/
            return 0;
        });
    }

    return THREAD_STORRAGES.with(|st| {
        let storrage: &Storrage = &(*st.borrow());
        let length = storrage.string_len;

        // Вариант без проверок границы
        unsafe{
            let bytes_slice: &[u8] = storrage.buffer.get_unchecked(0..length);
            if s2_test_str.eq(bytes_slice) {
                return 1;
            }
        }
        // Вызывается метод index из трейта Index, который возвращает ссылку и паникует при выходе за границы
        /*let ref bytes_slice = storrage.buffer[0..length];
        if s2_test_str.eq(bytes_slice) {
            return 1;
        }*/  
        // Способ ниже с проверкой выхода за границы, get выдает ссылку на слайс
        /*if let Some(bytes_slice) = storrage.buffer.get(0..length){
            if s2_test_str.eq(bytes_slice) {
                return 1;
            }   
        }*/
        return 0;
    });

    // Установка отлавливания паники слегка замедляет код, 
    // если нужна максимальная скорость - можно убрать
    // тогда производительность кода будет такая же
    /*let result = std::panic::catch_unwind(move || -> i32 {
    });

    // Обработка ошибки
    match result {
        Ok(res) =>{
            res
        }
        Err(_) =>{
            0
        }
    }*/
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
pub mod tests {
    use super::*;
    //use crate::my_test_functions::*;

    #[test]
    fn icmp1_test() {
        test_icmp();
    }
}
