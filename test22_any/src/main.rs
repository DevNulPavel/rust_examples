use std::fmt::Debug;
use std::any::{Any, TypeId};

// https://doc.rust-lang.org/book/appendix-02-operators.html
// Символ ? нужен, чтобы сказать, что тип может быть динамическикого размера
fn is_string<T: ?Sized + Any>(_s: &T) -> bool {
    TypeId::of::<String>() == TypeId::of::<T>()
}

// Функция, которая принимает любой объект, который реаллизует трейт Debug + Any
fn log<T: Any + Debug>(value: &T) {
    // Приводим к типу Any, dyn нужен так как мы не знаем точно размер объекта при компиляции
    let value_any = value as &dyn Any;

    // Пробуем сконвертировать наше значение в строку, если успешно - выводим строку и размер
    // если не вышло - выводим как есть с помощью трейта Debug
    match value_any.downcast_ref::<String>() {
        Some(as_string) => {
            println!("String ({}): {}", as_string.len(), as_string);
        }
        None => {
            println!("{:?}", value);
        }
    }
}

fn main() {
    // Можем работать с разными типами
    {
        let my_string = "Hello World".to_string();
        log(&my_string);
    
        let my_i8: i8 = 100;
        log(&my_i8);    
    }

    // Можем проверить тип
    assert_eq!(is_string(&0), false);
    assert_eq!(is_string(&"cookie monster".to_string()), true);

    // Можем вывести тип
    assert_eq!(
        std::any::type_name::<Option<String>>(),
        "core::option::Option<alloc::string::String>",
    );
}