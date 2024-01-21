use crate::{cat::Cat, dog::Dog, pet::Pet};
use std::os::raw::c_void;

////////////////////////////////////////////////////////////////////////////////

const POINTER_SIZE: usize = std::mem::size_of::<*const c_void>();

////////////////////////////////////////////////////////////////////////////////

/// Подробнее можно почитать здесь:
///     - https://doc.rust-lang.org/std/ptr/struct.DynMetadata.html
#[repr(C)]
#[derive(Copy, Clone)]
struct PetVtable {
    /// Указатель на деструктор для конкретного типа
    drop: fn(*mut c_void),

    /// Размер типа
    size: usize,

    /// Его выравнивание
    align: usize,

    /// Указатель на функцию конкретного типа
    sound: fn(*const c_void) -> String,

    /// Указатель на функцию конкретного типа
    name: fn(*const c_void) -> String,
}

////////////////////////////////////////////////////////////////////////////////

// Подменная функция
fn bark(_this: *const c_void) -> String {
    "Woof!".to_string()
}

////////////////////////////////////////////////////////////////////////////////

fn greet_pet(pet: Box<dyn Pet>) {
    println!("You: Hello, {}!", pet.name());
    println!("{}: {}\n", pet.name(), pet.sound());
}

////////////////////////////////////////////////////////////////////////////////

pub(super) fn test_v1() {
    unsafe {
        // Создадим объекты в куче, Box будет оборачивать Fat-указатели,
        // состоящие из двух частей:
        //      - указателя на сами данные
        //      - указатель на таблицу виртуальных методов
        let doggo: Box<dyn Pet> = Box::new(Dog::new("Doggo"));
        let mut kitty: Box<dyn Pet> = Box::new(Cat::new("Kitty"));

        // Получаем указатель на данные в виде просто числа
        let addr_of_data_ptr = ((kitty.as_mut() as *mut dyn Pet) as *mut c_void) as usize;
        dbg!(addr_of_data_ptr);

        // Сразу за указателем на данные у нас идет указатель на таблицу виртуальных методов,
        // данный указатель является общим для всех инстансов конкретного типа.
        // Получаем его в виде числа с помощью смещения от адреса данных.
        let addr_of_pointer_to_vtable = addr_of_data_ptr + POINTER_SIZE;
        dbg!(addr_of_pointer_to_vtable);

        // Конвертируем теперь этот адрес в мутабельный указатель на константную
        // таблицу виртуальных методов данного конкретного объекта.
        let ptr_to_ptr_to_vtable = addr_of_pointer_to_vtable as *mut *const PetVtable;

        // Создаем локальную копию таблицы виртуальных методов
        let mut new_vtable: PetVtable = (*ptr_to_ptr_to_vtable).read_unaligned();
        dbg!("New table copy");

        // В этой локальной копии мы подменяем указатель на функцию
        new_vtable.sound = bark;

        // Теперь для конкретного инстанса прописываем новую локальную таблицу виртуальных методов
        *ptr_to_ptr_to_vtable = &new_vtable;

        // Тестируем
        greet_pet(doggo);
        greet_pet(kitty);
    }
}
