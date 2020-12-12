use std::{
    any::{
        Any
    },
    fmt::{
        Debug
    }
};

// Логируем объект любого типа, которая реализует тип Debug + Any
fn log<T: Any + Debug>(value: &T) {
    // Приводим к конкретному типу
    let value_any: &dyn Any = value; // as &dyn Any

    let is_string = value_any.is::<String>();
    println!("Is string: {}", is_string);

    // Пытаемся сконвертировать наше значение в тип строки, если все прошло успешно
    // То помимо строки, выводим еще ее длину
    match value_any.downcast_ref::<String>() {
        Some(as_string) => {
            println!("String ({}): {}", as_string.len(), as_string);
        }
        None => {
            println!("{:?}", value);
        }
    }
}

// Данная функция принимает объект, который реализует трейт Any + Debug
fn do_work<T: Any + Debug>(value: &T) {
    log(value);
    // ...do some other work
}

fn main() {
    let my_string = "Hello World".to_string();
    do_work(&my_string);

    let my_i8: i8 = 100;
    do_work(&my_i8);
}