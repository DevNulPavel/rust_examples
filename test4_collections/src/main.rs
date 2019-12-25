#![allow(unused_imports, dead_code, unused_variables)]

use std::collections::HashMap;


fn test_vector(){
    // Создание вектора типа i32
    let v: Vec<i32> = Vec::new();
    print!("{:?} ", v);

    let v: Vec<i32> = vec![1, 2, 3];
    println!("{:?}",v);

    let mut v = vec![1, 2, (3 as i32)];
    println!("{:?}",v);

    v.push(3);
    v.push(4);

    println!("{:?}",v);

    // Получаем ссылку на 3й элемент, но данный подход не очень правильный
    // так как не происходит проверки невалидного индекса,
    // как результат - можно получить ошибку в рантайме, зато способ быстрый
    let third_element: &i32 = &v[2]; 
    println!("{:?}", third_element);

    // Данный способ получения значительно лучше, так как
    // тип Option позволяет проверить выход за границы массива
    let third_element: Option<&i32> = v.get(10); // Получаем ссылку на 10й элемент
    println!("{:?}", third_element);

    let mut v1 = vec![1, 2, 3, 4, 5];
    let first = &v1[0];
    v1.push(6);
}

fn test_iterators() {
    // Данным способом мы итерируемся, но элементы у нас неизменяемы, просто ссылки на них?
    let names = vec!["Bob", "Frank", "Ferris"];
    for name in names.iter() {
        match name {
            // Поэтому здесь указана ссылка
            &"Ferris" => println!("Программисты Rust вокруг нас!"),
            _ => println!("Привет {}", name),
        }
    }
    println!("имена: {:?}", names);

    // Здесь мы итерируемся уже с помощью изменяемого итератора - содержимое уже не будет доступно
    // println!("имена: {:?}", names); - выдаст ошибку
    let names = vec!["Bob", "Frank", "Ferris"];
    for name in names.into_iter() {
        // Значения перемещаются, поэтому тут нету никакой ссылки
        match name {
            "Ferris" => println!("Программисты Rust вокруг нас!"),
            _ => println!("Привет {}", name),
        }
    }

    //
    let mut names = vec!["1", "2", "3"];
    for name in names.iter_mut() {
        // Мы можем модифицировать содержимое по ссылке
        *name = match name {
            &mut "3" => "30",
            _ => "0",
        }
    }
    println!("имена: {:?}", names);
}

fn test_strings(){
    let s1 = String::from("Hello ");
    let s2 = String::from("world");
    let s3 = String::from("!");
    // Складывание строк должно происходить обязательно со слайсом
    // для складывания строки реализован метод:
    // fn add(self, s: &str) -> String {
    let s4 = s1.clone() + &s2 + &s3; 
    println!("Result string: {}", s4);

    // Так же имеется возможность использовать макрос format!
    let s5 = format!("{}{}{}", s1, s2, s3);
    println!("Result string: {}", s5);

    //let h = s1[0]; // У сток мы не можем поиндексно получать символы
    let h = s1.as_bytes()[0]; // Но можем по байтам
    println!("{}", h);

    // Тип String это объертка Vec<u8>
    let len = String::from("Hola").len();
    println!("{}", len);
    let len = String::from("Русский текст").len(); // Каждый символ кодируется уже 2 байтими
    println!("{}", len);

    let hello = "Здравствуйте";
    //let answer = &hello[0]; // По этой причине запрещено делать так
    let s1 = &hello[0..4]; // Однако можно получать побайтовые слайсы от строк, но чтобы попадали обязательно символы
    println!("{}", s1);
    //let s2 = &hello[0..1]; // Однако это приведет к ошибке в рантайме, так как не цепляем символ

    let chars = "नमस्ते".chars();
    for c in chars {
        println!("{}", c);
    }
    for c in "नमस्ते".bytes() {
        println!("{}", c);
    }
}

fn test_hash_maps(){
    use std::collections::HashMap;

    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);
    println!("{:?}", scores);

    let teams = vec![String::from("Blue"), String::from("Yellow")];
    let initial_scores = vec![10, 50];
    // Такой необычный тип данных HashMap<_, _> необходим, 
    // т.к. метод collect может содержать данные разных типов и Rust 
    // не может заранее проверить их соответствие.
    let scores: HashMap<_, _> = teams.iter().zip(initial_scores.iter()).collect();
    println!("{:?}", scores);

    let field_name = String::from("Favorite color");
    let field_value = String::from("Blue");
    let mut map = HashMap::new();
    map.insert(field_name, field_value);
    println!("{:?}", map);

    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);
    let team_name = String::from("Blue");
    let score = scores.get(&team_name);
    println!("{} {:?}", team_name, score);

    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);
    scores.insert(String::from("Yellow"), 50);
    scores.insert(String::from("Yellow"), 60); // Значение будет перезаписано
    for (key, value) in &scores {
        println!("{}: {}", key, value);
    }

    let mut scores = HashMap::new();
    scores.insert(String::from("Blue"), 10);
    scores.entry(String::from("Yellow")).or_insert(50); // Если данных у нас там нету, то вставляем
    scores.entry(String::from("Blue")).or_insert(50);   // Если данных у нас там нету, то вставляем
    println!("{:?}", scores);
}

fn main() {
    //test_vector();
    test_iterators();
    //test_strings();
    //test_hash_maps();
}
