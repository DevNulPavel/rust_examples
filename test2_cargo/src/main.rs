// Таким образом можно указать для всего файла (#!) отключение неиспользуемых переменных
#![allow(unused_variables, unused_mut, dead_code)]

extern crate rand;

use std::io; // Можно в коде использовть напрямую std::io::
use std::cmp::Ordering;
use rand::Rng;

fn test_generator(){
    let mut random_generator = rand::thread_rng();
    let random_value: u32 = random_generator.gen_range(0, 10);

    loop {
        println!("Загадайте число от 0 до 9 и введите его:");

        // Создаем изменяемую переменную
        let mut input_text: String = String::new();

        // Создается новый экземпляр-обработчик ввода
        let stdin = io::stdin();

        // Читаем значение из стандартного ввода, передаем изменяемую ссылку
        let read_result = stdin.read_line(&mut input_text);

        // Парсим данную строку в число
        //let read_result_number: u32 = input_text.trim().parse().expect("Ошибка парсинга");
        
        // Заменяем вызов метода expect на выражение match
        // таким образом при ошибке мы можем заново начать цикл
        let read_result_number: u32 = match input_text.trim().parse() {
            Ok(num) => num,
            Err(_) => continue,
        };

        // Если рузультат у нас будет ошибкой, то этот метод завершит работу и выведет ошибку ниже,
        // если результат будет нормальный, то вернется результат работы.
        // Вызов expect является обязательным, так как мы обязательно должны обработать ошибку
        let number_of_bytes = read_result.expect("Не получилось прочитать строку");

        println!("Вы загадали: {} ({} bytes)", input_text, number_of_bytes);

        // Пробуем сравнить значение больше оно или нет
        match read_result_number.cmp(&random_value){
            Ordering::Less    => {
                println!("Too small!");
            }
            Ordering::Greater => { 
                println!("Too big!");
            }
            Ordering::Equal   => {
                println!("You win!");
                break;
            }
        }
    }
}

// Тестировании области видимости констант
const MAX_POINTS: u32 = 200_000;

fn print_constants(){
    println!("MAX_POINTS is: {}", MAX_POINTS);
}

fn test_constants(){
    println!("MAX_POINTS is: {}", MAX_POINTS);
    const MAX_POINTS: u32 = 100_000;
    print_constants();
}

fn test_variables(){
    // В Rust можно задавать переменным одно и то же имя в пределах 
    // одной области видимости, используется всегда последняя
    let x:i32 = 5i32; // Можно указывать конкретный тип у значения
    let x:u64 = 5u64;
    let x = x+1;
    let x = x*2;
    println!("The value of x is: {}", x);

    // А вот для изменяемых переменных такое поведение запрещено
    /*let mut y = "_";
    println!("The value of x is: {}", x);
    spaces = y.len();
    println!("The value of x is: {}", x);*/

    let tuple: (i32, f64) = (30, 50.0);
    let (x, y) = tuple;
    println!("Tuple = {:?}", tuple);
    let x1 = tuple.0;
    let y1 = tuple.1;

    let array1 = [1, 2, 3, 4, 5];
    println!("Array 1 = {:?}", array1);
    let array2: [u32; 5] = [1, 2, 3, 4, 5];
    println!("Array 2 = {:?}", array2);
    println!("Array 2 at 1 = {}", array2[1]);
    //println!("Array 1 at 10 = {}", array1[10]); // Будет ошибка, размер не совпадает!
}

fn test_function(val1: i32, val2: String) -> u32 {
    println!("This is value {}, this is string {}", val1, val2);

    if val1 <= 10 {
        println!("First parameter is less or eq 10");
    }else{
        println!("First parameter is greater 10");
    }

    let test_val = if val1 > 10 { 5 } else { 6 };
    println!("Test value is {}", test_val);

    // Пример итерирования по массиву
    let test_array = [10, 20, 30, 40, 50];
    for element in test_array.iter() {
        println!("The value is: {}", element);
    }

    // Пример итерирования по числам
    for i in 0..10 {
        println!("Index i = {}", i);
    }

    return 10;
}

// Когда вызывается данная функция, то владение параметром строки переходит к ней
fn take_ownership(some_string: String){
    println!("{}", some_string);
    // На выходе из функции произойдет уничтожение строки
}

// Для переменных числовых используется копирование значений
fn makes_copy(some_integer: i32) {
    println!("{}", some_integer);
}

fn gives_ownership(input: String)-> String{
    println!("{}", input);
    return input;
}

// Однако можно без проблем принимать ссылку на строку, тогда не происходит перемещения владения
fn takes_string_link(input: &mut String){
    input.push_str("test");
    println!("{}", input);
}

fn first_word_index(s: &String) -> usize{
    // Получаем в виде слайса байт
    let bytes: &[u8] = s.as_bytes();

    // Итерирование по каждому байту
    for (i, &item) in bytes.iter().enumerate() {
        // Как только мы попадаем на пробел - значит нашли конец слова
        if item == b' '{
            return i;
        }
    }

    // Не нашли - конец
    return s.len();
}

fn first_word_slice(s: &str) -> &str{
    // Получаем в виде слайса байт
    let bytes: &[u8] = s.as_bytes();

    // Итерирование по каждому байту
    for (i, &item) in bytes.iter().enumerate() {
        // Как только мы попадаем на пробел - значит нашли конец слова
        if item == b' '{
            return &s[0..i];
        }
    }

    // Не нашли - конец
    return s;
}

fn borrow_test(){
    let s1: &str = "Test string 1"; // Ссылка на статическую память, по сути - стринг вью
    let s1_another: &str = s1; // Данный код является валидным, можно иметь несколько стринг вью на статические данные
    let s2: String = String::from("Test string 2"); // Строка в куче
    let mut s3: String = String::from(s1_another); // Строка в куче изменяемая, которая является копией слайса

    s3.push_str(" another text");

    println!("{}", s3);

    let s4 = s3; // После такого вызова переменная s3 становится невалидной
    //println!("{}", s3); // Такой код не сработает
    println!("{}", s4);

    let s5 = s4.clone(); // Создание копии
    println!("{} {}", s4, s5); // Данный код является валидным

    take_ownership(s5); // Владение переменной перемещается в функцию
    //println!("{}", s5); // Здесь мы уже не можем использовать эту переменную, так как она уничтожена

    let x: i32 = 10;
    makes_copy(x);      // При копировании переменных из стека проблем не возникает
    println!("{}", x);  // с последующим использованием

    let s6: String = String::from("Test");
    let mut s6: String = gives_ownership(s6); // Однако всегда можно вернуть владение объектом
    println!("{}", s6);  // с последующим использованием

    takes_string_link(&mut s6); // Однако можно без проблем принимать ссылку на строку, тогда не происходит перемещения владения

    let s7: String = String::from("This is test text");
    let first_word_index: usize = first_word_index(&s7);

    // При работе со слайсами надо быть аккуратным, так как
    // индексы можно брать совершенно любые
    let slice1: &str = &s7[0..5];
    // let slice2: &str = &s7[10..20]; // Тут будет ошибка в рантайме
    let slice3: &str = &s7[..5]; // Можно опустить начало
    let slice4: &str = &s7[..]; // Можно опустить начало и конец
    let slice5: &str = &s7[2..]; // Можно опустить конец

    let slice6: &str = first_word_slice(&s7[..]); // Можно передавать в функцию слайс + получаем слайс

    let array: [u32; 5] = [1, 2, 3, 4, 5];
    let array_slice: &[u32] = &array[2..4];
    println!("Slice {:#?}", array_slice);
}

#[derive(Debug)] // Clone,Copy // Этой строчкой мы включаем отладочный вывод для структур с помощью {:?}
struct User{
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool
}

impl Clone for User{
    fn clone(&self) -> User{
        let new_user = User{
            username: self.username.clone(),
            email: self.email.clone(),
            sign_in_count: self.sign_in_count,
            active: self.active,
        };
        return new_user;
    }
}

impl User{
    fn get_username(&self) -> &str{
        return &self.username[..];
    }
    fn get_email(&self) -> &str{
        return &self.email[..];
    }
    fn set_email(&mut self, new_email: &str){
        self.email = String::from(new_email);
    }
}

fn make_user(name: String, email: String) -> User {
    let user = User {
        username: name,
        email: email,
        sign_in_count: 1,
        active: true
    };
    return user;
}

fn test_structures() {
    let user: User = make_user(String::from("name"), String::from("email"));
    println!("User data: {:?}", user);
}

enum IpAddrKind{
    V4,
    V6
}

enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}

enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write(String),
    ChangeColor(i32, i32, i32),
}

impl Message{
    fn call(&self){
    }
}

// Определено в стандартной библиотеке
// enum Option<T> {
//     Some(T),
//     None,
// }

fn test_enum() {
    let home = IpAddr::V4(127, 0, 0, 1);
    let loopback = IpAddr::V6(String::from("::1"));
    let some_number = Some(5);
    let some_string = Some("String");
    let none_number: Option<i32> = None;

    // TODO: ???
    //let summ_value = some_number + none_number
}

enum Coin {
    Penny,
    Nickel,
    Dime,
    Quarter,
    Unknown,
}

fn test_match(){
    let coin_val: Coin = Coin::Quarter;
    let value = match coin_val {
        Coin::Penny => 1,
        Coin::Nickel => 5,
        Coin::Dime => 10,
        Coin::Quarter => 25,
        _ => 100
    };
}

fn main() {
    //test_generator();
    //test_constants();
    //test_variables();
    //test_function(10, "test text".to_string());
    //borrow_test();
    //test_structures();
    test_match();
}
