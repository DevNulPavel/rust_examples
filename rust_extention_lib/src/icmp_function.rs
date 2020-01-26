
use std::ffi::CStr;
use std::os::raw::c_char;
use std::cell::RefCell;

// https://doc.rust-lang.org/nomicon/ffi.html

#[no_mangle] 
pub extern "C" fn icmp1_RUST_CODE(s2: *const c_char, s1: *const c_char) -> i32 {
    // Установка отлавливания паники слегка замедляет код, 
    // если нужна максимальная скорость - можно убрать
    // тогда производительность кода будет такая же
    let result = std::panic::catch_unwind(move || -> i32 {
        if s2.is_null() {
            return 0;
        }
    
        // Создаем Rust ссылочную строку из С-шной
        // Если не смогли сконвертить в Rust строку - выходим
        let s2_test_str = unsafe { CStr::from_ptr(s2) }.to_bytes();
    
        // Защита от кривых данных
        if s2_test_str.len() > 1024{
            return 0;
        }
    
        struct Storrage{
            buffer: Vec<u8>,
            string_len: usize,
        };
        impl<'a> Storrage{
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
            let res = unsafe { CStr::from_ptr(s1) }.to_bytes();
    
            // Защита от кривых данных
            if res.len() > 1024{
                return 0;
            }
    
            THREAD_STORRAGES.with(|st| {
                let storrage: &mut Storrage = &mut *st.borrow_mut();
    
                // Увеличиваем размер буффера
                if storrage.buffer.len() < res.len(){
                    storrage.buffer.resize(res.len(), 0);
                }
    
                // Переводим в нижний регистр
                let len = res.len();
                for i in 0..len {
                    let byte = res[i];
                    let lowercase_byte = byte.to_ascii_lowercase();
                    storrage.buffer[i] = lowercase_byte;
                }
                storrage.string_len = res.len();
            });
        }
    
        return THREAD_STORRAGES.with(|st| {
            let storrage: &Storrage = &(*st.borrow());
            let length = storrage.string_len;
    
            // Такой способ лучше, так как он не приводит к копированию
            if let Some(bytes_slice) = storrage.buffer.get(0..length){
                if s2_test_str.eq(bytes_slice) {
                    return 1;
                }else{
                    return 0;
                }    
            }else{
                return 0;
            }
        });
    });

    // Обработка ошибки
    match result {
        Ok(res) =>{
            res
        }
        Err(_) =>{
            0
        }
    }
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
