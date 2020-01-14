#![allow(unused_variables)]

extern crate libc;


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
    {
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
    }

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

#[cfg(test)]
mod tests {
    use crate::my_test_functions::*;

    #[test]
    fn test() {
        test_raw_pointers();
    }
}