#[allow(unused_assignments)]


fn main() {
    println!("Hello, Rust world! Привет, Мир!");
    let mut test: i32 = 120; // Тип можно указывать перед переменной
    test += 40;
    let mut result = Some(test);

    let robot_name = Some(String::from("Bors"));
    match robot_name {
        // Пример не соберется без ref, так как строка переместит свое владение внутрь блока
        // ref позволяет получить ссылку на значение
        Some(ref name) => {
            println!("Found a name: {}", name);
        },
        None => (),
    }
    println!("robot_name is: {:?}", robot_name);

    // Для модифицирования значения внутри Some нужно использовать ref mut, 
    // он делает мутабельную ссылку на значение, которое можно менять при разыменовании
    match result {
        // Мы можем дополнительно добавлять всякие разные проверки в блок match
        // https://doc.rust-lang.ru/book/ch18-03-pattern-syntax.html
        Some(ref mut val) if *val < 1000 =>{
            *val += 1;
        },
        Some(ref mut val)=>{
            *val += 10;
        },
        None => ()
    }
    println!("{:?}", result);
}