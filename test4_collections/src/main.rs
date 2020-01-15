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

fn test_slices(){
    // https://doc.rust-lang.org/std/primitive.slice.html

    // Слайс представляет из себя толстый указатель, то есть обычный указатель + длина
    {
        // Создаем вектор, данные лежат в куче
        let vec = vec![1, 2, 3];
        // Получаем слайс на эти данные
        let int_slice = &vec[..];
        println!("{:?}", int_slice);

        // Создаем слайс на массив, массив и строки из него лежат в константной памяти
        let str_slice: &[&str] = &["one", "two", "three"];
        println!("{:?}", str_slice);
    }

    {
        let mut x = [1, 2, 3];
        // У нас может быть одна мутабельная ссылка-слайс на массив
        let x = &mut x[..];
        x[1] = 7;
        assert_eq!(x, &[1, 7, 3]);
    }

    {
        let a = [1, 2, 3];
     
        // У слайса есть метод получения длины
        assert_eq!(a.len(), 3);

        // Проверка, что он не пустой
        assert!(!a.is_empty());

        // Также мы можем получить первый элемент
        assert_eq!(Some(&1), a.first()); // TODO: &1 - так как число 1 только одно в константной памяти, поэтому и ссылка только одна???
        assert_ne!(None, a.first());
    }

    {
        let x = &mut [0, 1, 2];

        // Мы можем получить мутабельную ссылку на первый элемент
        if let Some(first) = x.first_mut() {
            *first = 5;
        }
        assert_eq!(x, &[5, 1, 2]);
    }

    {
        let x = &[0, 1, 2];

        // Можем разделить слайс на первый элемент и на слайс оставшихся
        // есть аналогичный вариант с last
        if let Some((first, elements)) = x.split_first() {
            assert_eq!(first, &0);
            assert_eq!(elements, &[1, 2]);
        }
    }

    // Можно разделять слайс на нужной позиции, а не только в начеле и конце
    {
        let v = [1, 2, 3, 4, 5, 6];

        {
            let (left, right) = v.split_at(0);
            assert!(left == []);
            assert!(right == [1, 2, 3, 4, 5, 6]);
        }

        {
            let (left, right) = v.split_at(2);
            assert!(left == [1, 2]);
            assert!(right == [3, 4, 5, 6]);
        }

        {
            let (left, right) = v.split_at(6);
            assert!(left == [1, 2, 3, 4, 5, 6]);
            assert!(right == []);
        }
    }

    {
        let slice = [10, 40, 33, 20];

        // Мы можем создавать новые слайсы с помощью предикатов
        // если предикат возвращает true - значит элемент не попадает в итератор
        let mut iter = slice.split(|num| {
            num % 3 == 0
        });
        
        assert_eq!(iter.next().unwrap(), &[10, 40]);
        assert_eq!(iter.next().unwrap(), &[20]);
        assert!(iter.next().is_none());    
    }

    {
        let v = [10, 40, 30];
        // Можно получить конкретное значение из слайса
        assert_eq!(Some(&40), v.get(1));
        // Либо мы можем получить подслайс с нужными значениями
        assert_eq!(Some(&[10, 40][..]), v.get(0..2));
        // Если элемента нету - значит возращается None
        assert_eq!(None, v.get(3));
        assert_eq!(None, v.get(0..4));
    }

    {
        // Можем поменять местами значения
        let mut v = ["a", "b", "c", "d"];
        v.swap(1, 3);
        assert!(v == ["a", "d", "c", "b"]);
    }

    {
        // Можно даже создать слайс с обратным порядком
        let mut v = [1, 2, 3];
        v.reverse();
        assert!(v == [3, 2, 1]);
    }

    {
        // Window метод возвращает итератор, где шагом итерации будет смещение на 1н элемент
        // но при этом - возвращаться будет по 2 элемента
        let slice = ['r', 'u', 's', 't'];
        let mut iter = slice.windows(2);
        assert_eq!(iter.next().unwrap(), &['r', 'u']);
        assert_eq!(iter.next().unwrap(), &['u', 's']);
        assert_eq!(iter.next().unwrap(), &['s', 't']);
        assert!(iter.next().is_none());
    }

    {
        // Однако есть другой вариант - можно возращать значения из слайса группами по 
        // n элементов
        let slice = ['l', 'o', 'r', 'e', 'm'];
        let mut iter = slice.chunks(2);
        assert_eq!(iter.next().unwrap(), &['l', 'o']);
        assert_eq!(iter.next().unwrap(), &['r', 'e']);
        assert_eq!(iter.next().unwrap(), &['m']);
        assert!(iter.next().is_none());
    }

    {
        let v = &mut [0, 0, 0, 0, 0];
        let mut count = 1;

        // Либо можно возращать изменяемые подслайсы
        for chunk in v.chunks_mut(2) {
            for elem in chunk.iter_mut() {
                *elem += count;
            }
            count += 1;
        }
        assert_eq!(v, &[1, 1, 2, 2, 3]);
    }

    {
        // Можно возвращать подслайсы только если осталось нужное количество элементов
        // Есть аналогичный вариант с мутабельным слайсом
        let slice = ['l', 'o', 'r', 'e', 'm'];
        let mut iter = slice.chunks_exact(2);
        assert_eq!(iter.next().unwrap(), &['l', 'o']);
        assert_eq!(iter.next().unwrap(), &['r', 'e']);
        assert!(iter.next().is_none());
        assert_eq!(iter.remainder(), &['m']);
    }

    {
        // Есть аналогичный метод, но возвращающий чанки с конца
        let slice = ['l', 'o', 'r', 'e', 'm'];
        let mut iter = slice.rchunks(2);
        assert_eq!(iter.next().unwrap(), &['e', 'm']);
        assert_eq!(iter.next().unwrap(), &['o', 'r']);
        assert_eq!(iter.next().unwrap(), &['l']);
        assert!(iter.next().is_none());
    }

    {
        // Можно проверить, содержится ли нужный элемент в слайсе
        let v = [10, 40, 30];
        assert!(v.contains(&30));
        assert!(!v.contains(&50));
    }

    {
        // Можно проверить, что слайс начинается и заканчивается чем-то
        let v = [10, 40, 30];
        assert!(v.starts_with(&[10]));
        assert!(v.starts_with(&[10, 40]));
        assert!(!v.starts_with(&[50]));
        assert!(!v.starts_with(&[10, 50]));
    }

    {
        // Можно даже выполнять двоичный поиск по отсортированному массиву

        let s = [0, 1, 1, 1, 1, 2, 3, 5, 8, 13, 21, 34, 55];

        assert_eq!(s.binary_search(&13),  Ok(9));
        assert_eq!(s.binary_search(&4),   Err(7));
        assert_eq!(s.binary_search(&100), Err(13));
        let r = s.binary_search(&1);
        assert!(match r { Ok(1..=4) => true, _ => false, });
    }

    {
        // Можно выполнять сортировку без сохранения порядка одинаковых элементов
        let mut v = [-5, 4, 1, -3, 2];

        v.sort_unstable();
        assert!(v == [-5, -3, 1, 2, 4]);
    }

    {
        // Обычная стабильная сортировка с сохранением порядка одинаковых элементов
        let mut v = [-5, 4, 1, -3, 2];

        v.sort();
        assert!(v == [-5, -3, 1, 2, 4]);        
    }

    {
        // Можно выполнять циклическое смещение элементов на нужное значение влево с перемещением их в конец
        let mut a = ['a', 'b', 'c', 'd', 'e', 'f'];
        a.rotate_left(2);
        assert_eq!(a, ['c', 'd', 'e', 'f', 'a', 'b']);
    }

    {
        // Можно копировать значения из одного слайса в другой
        let src = [1, 2, 3, 4];
        let mut dst = [0, 0];

        // Размерность слайсов должна быть одинаковая
        dst.copy_from_slice(&src[2..]);

        assert_eq!(src, [1, 2, 3, 4]);
        assert_eq!(dst, [3, 4]);
    }

    {
        // Можно конвертировать слайс в вектор
        let s = [10, 40, 30];
        let x = s.to_vec();
    }
    
    {
        // Box можно конвертировать в вектор с перемещением данных, то есть без копирования
        let s: Box<[i32]> = Box::new([10, 40, 30]);
        let x = s.into_vec();
        // s больше нельзя ипользовать, он перенесен в x

        assert_eq!(x, vec![10, 40, 30]);
    }

    {
        // Можно производить объединение слайсов в общее значение
        assert_eq!(["hello", "world"].concat(), "helloworld");
        assert_eq!([[1, 2], [3, 4]].concat(), [1, 2, 3, 4]);
    }

    {
        // Можно производить соединение с использованием разделителя
        assert_eq!(["hello", "world"].join(" "), "hello world");
        assert_eq!([[1, 2], [3, 4]].join(&0), [1, 2, 0, 3, 4]);
        assert_eq!([[1, 2], [3, 4]].join(&[0, 0][..]), [1, 2, 0, 0, 3, 4]);
    }
}

fn main() {
    //test_vector();
    // test_iterators();
    //test_strings();
    //test_hash_maps();
    test_slices();
}
