#![allow(dead_code)]

use std::thread;
use std::time::Duration;
use std::collections::HashMap;

fn test_closure(){
    // В качестве параметра можно указывать
    let expensive_closure = |number: i32| -> i32 {
        println!("calculating slowly...");
        thread::sleep(Duration::from_secs(2));
        number
    };
    expensive_closure(5);
}

// Определяем структуру, шаблонный параметр - это функция, принимающия число и возвращяющая его
// Есть несколько типов типпажей - FnOnce, Fn, FnMut
// - FnOnce получает значения из области видимости (environment). Для получения доступа к переменным замыкание должно получить во владения используемые переменные. Замыкание не может получить во владение одну и туже переменную несколько раз.
// - Fn заимствует значения из среды (не изменяя при этом их значений).
// - FnMut может изменять значения переменных.
struct Cacher<T> where T: Fn(i32) -> i32 {
    calculation_func: T,
    values: HashMap<i32, i32>,
}

// Реализуем код структурки
impl<T> Cacher<T> where T: Fn(i32) -> i32{
    fn new(calculation_func: T) -> Cacher<T> {
        Cacher {
            calculation_func,
            values: HashMap::new(),
        }
    }

    fn value(&mut self, arg: i32) -> i32 {
        // Смотрим, есть ли у нас значение
        match self.values.get(&arg) {
            // Если значение есть - отдаем
            Some(v) => {
                // Возврат копии
                *v
            },
            // Если нету - вычисляем
            None => {
                // Вычисляем значение
                let v = (self.calculation_func)(arg);
                // Сохраняем в нашу таблицу
                self.values.insert(arg, v);
                // Возвращаем значение
                v
            },
        }
    }
}

fn generate_workout(intensity: i32, random_number: i32) {
    // Создаем кеширующий объект
    let mut expensive_result = Cacher::new(|num| {
        println!("calculating slowly...");
        thread::sleep(Duration::from_secs(2));
        num
    });

    if intensity < 25 {
        println!(
            "Today, do {} pushups!",
            expensive_result.value(intensity) // Получаем значение, которое считаем
        );
        println!(
            "Next, do {} situps!",
            expensive_result.value(intensity) // Получаем значение, которое мы закешировали
        );
    } else {
        if random_number == 3 {
            println!("Take a break today! Remember to stay hydrated!");
        } else {
            println!(
                "Today, run for {} minutes!",
                expensive_result.value(intensity) // Получаем значение, которое считаем
            )
        }
    }
}

fn move_data_to_lambda(){
    // Создаем вектор с данными
    let x = vec![1, 2, 3];

    // Говорим, что владение переменной должно переместиться внутрь лямбды
    let equal_to_x_lambda = move |z| { 
        return z == x; 
    };

    // Однако, можно функцию прямо здесь определять
    /*fn equal_to_x(z: i32) -> bool { 
        z == x 
    };*/

    // Соответственно - код ниже уже невалидный
    // println!("can't use x here: {:?}", x);
    
    let y = vec![1, 2, 3];
    assert!(equal_to_x_lambda(y));
}

fn test_iterators(){
    let v1 = vec![1, 2, 3];
    // Итератор реализует trait Iterator + метод next
    /*trait Iterator {
        type Item; // Тип, ассоциированный с типажом
        fn next(&mut self) -> Option<Self::Item>; // Self - текущий класс
    }*/

    let v1_iter = v1.iter();
    for val in v1_iter {
        println!("Got: {}", val);
    }
}

struct Counter {
    count: u32,
}

impl Counter {
    fn new() -> Counter {
        Counter { count: 0 }
    }
}

// Таким образом мы реализуем наш итератор для счетчика
impl Iterator for Counter {
    type Item = u32;

    // Возвращаем Option
    fn next(&mut self) -> Option<Self::Item> {
        self.count += 1;

        if self.count < 6 {
            Some(self.count)
        } else {
            None
        }
    }
}

fn test_lambdas(){
    // Функция, которая принимает замыкание в качестве аргумента и вызывает его.
    // <F> обозначает, что F - "параметр общего типа"
    fn apply<F>(f: F) where F: FnOnce() { // Замыкание ничего не принимает и не возвращает.
        // ^ TODO: Попробуйте изменить это на `Fn` или `FnMut`.
        f();
    }

    // Функция, которая принимает замыкание и возвращает `i32`.
    fn apply_to_3<F>(f: F) -> i32 where F: Fn(i32) -> i32 { // Замыкание принимает `i32` и возвращает `i32`.
        f(3)
    }

    use std::mem;

    let greeting = "привет";
    // Не копируемый тип.
    // `to_owned` преобразует заимствованные данные в собственные.
    let mut farewell = "пока".to_owned();

    // Захват двух переменных: `greeting` по ссылке и
    // `farewell` по значению.
    let diary = || {
        // `greeting` захватывается по ссылке: требует `Fn`.
        println!("Я сказал {}.", greeting);

        // Изменяемость требует от `farewell` быть захваченным
        // по изменяемой ссылке. Сейчас требуется `FnMut`.
        farewell.push_str("!!!");
        println!("Потом я закричал {}.", farewell);
        println!("Теперь я могу поспать. zzzzz");

        // Ручной вызов удаления требуется от `farewell`
        // быть захваченным по значению. Теперь требуется `FnOnce`.
        mem::drop(farewell);
    };

    // Вызов функции, которая выполняет замыкание.
    apply(diary); // Аналогично diary();

    // `double` удовлетворяет ограничениям типажа `apply_to_3`
    let double = |x| {
        2 * x
    };
    println!("Удвоенное 3: {}", apply_to_3(double));
}

fn test_return_lambdas(){
    // Для возвращаемых лямбд нужно возвращать что-то, что реализует Fn,
    // для этого мы прописываем impl, так как мы возвращаем что-то, что реализует интерфейс
    fn create_fn() -> impl Fn() {
        // Создаем владеющую строку, которая владеет новым значением строки
        let text: String = "Fn".to_owned();
        // Создаем лямбду и закидываем туда экземпляр нашей строки
        let lambda = move || {
            println!("a: {}", text);
            // Так как мы должны иметь возможность вызывать лямбду несколько раз - нельзя уничтожать
            //drop(text);
        };
        return lambda;
    }
    
    fn create_fnmut() -> impl FnMut() {
        // Создаем изменяемую переменную
        let mut text = "FnMut".to_owned();
        // Перемещаем данные в замыкание
        let lambda = move || {
            // Можем изменять значение
            text.push_str("-changed");
            println!("a: {}", text);
        };
        return lambda;
    }
    
    fn create_fnonce() -> impl FnOnce() {
        let text = "FnOnce".to_owned();
        let lambda = move || {
            println!("a: {}", text);
            // Так как мы возвращаем лямбду FnOnce, можно спокойно уничтожать все
            drop(text);
        };
        return lambda;
    }

    let fn_plain = create_fn();
    let mut fn_mut = create_fnmut();
    let fn_once = create_fnonce();

    fn_plain();
    fn_plain();

    fn_mut();
    fn_mut();

    fn_once(); // Такую функцию можно вызывать лишь один раз, так как значения после становятся невалидными
}

fn main() {
    //test_lambdas();
    test_return_lambdas();
}