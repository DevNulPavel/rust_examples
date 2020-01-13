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

// Выстовляем соглашение о вызовах C + отключаем изменение имени функции
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


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}