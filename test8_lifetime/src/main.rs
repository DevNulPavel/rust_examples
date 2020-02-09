#![allow(unused_variables)]
#![allow(dead_code)]
// Таким образом можно указать для всего файла (#!) отключение неиспользуемых переменных #![allow(unused_variables, unused_mut, dead_code)]


use std::rc::Rc;
use std::cell::RefCell;

// https://habr.com/ru/post/442962/

// Функция с указанием конкретного времени жизни ссылок,
// тут указано, что все переменные должны иметь одно и то же время жизни
// Шаблонный параметр говорит, что у всех параметров должно быть одно время жизни
// обычно применяется имя a, но мы можем использовать любое другое название для времени жизни
fn longest_lifetime<'same>(x: &'same str, y: &'same str) -> &'same str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}

// Обычный вариант не соберется, так как время жизни ссылки неизвестно
/*fn longest(x: &str, y: &str) -> &str {
    if x.len() > y.len() {
        x
    } else {
        y
    }
}*/

fn test_lifetime(){
    // Такая переменная имеет статическое время жизни
    let s: &'static str = "I have a static lifetime.";

    let string1 = String::from("long string is long");

    {
        let string2 = String::from("xyz");
        //let result = longest(string1.as_str(), string2.as_str());
        let result = longest_lifetime(s, string2.as_str()); // у результата время жизни - a'
        println!("The longest string is {}", result);
    }
}

fn test_func(){
    // Макрос-предупреждение, что не реализована функция
    // паникует при вызове
    unimplemented!();
}

// Указываем время жизни, все ок (в шаблоне пишем, что все значения имеют время жизни 'a)
// Это значит, что структура должна жить столько же, сколько живут все объекты у структуры?
struct ImportantExcerpt<'a, 'b> {
    part1: &'a str,
    part2: &'b str,
}

fn test_struct(){
    let novel: String = String::from("Call me Ishmael. Some years ago...");
    // next возвращает первый элемент итератора
    let mut iter = novel.split('.');
    let first_sentence: &str = iter.next().expect("Could not find a '.'"); 
    let second_sentence: &str = iter.next().expect("Could not find a '.'"); 
    let object = ImportantExcerpt{ 
        part1: first_sentence,
        part2: second_sentence
    };
    println!("{}{}", object.part1, object.part2);
}

fn test_ref_cell(){
    // "a" начинает жить тут, объект немутабельный, но мы можем получить мутабельное содержимое в единственном экземпляре
    let a = RefCell::new("a".to_string());

    // замыкание одалживает немутабельный "a"
    let f_1 = || {
        // получаем мутабельный экземпляр содержимого и меняем его
        a.borrow_mut().push_str(" and x"); 
    };

    // ещё раз одалживает
    let f_2 = || {
        // получаем мутабельный экземпляр содержимого и меняем его
        a.borrow_mut().push_str(" and y");
    };

    // во время выполнения данной лямбды a.borrow_mut() видит, что больше никто не делает тоже самое и позволяет использовать mut значение в блоке лямбды,
    // если бы кто-то начал использовать, то была бы паника
    f_1();

    // аналогично для второй.
    f_2();

    // в данном случая для вывода нам не нужна мутабельность.
    println!("{}", a.borrow());
}

// именно String, а не одалживание &String
fn test_ref_func(x: Rc<RefCell<String>>) {
    // Со строкой можно работать как есть, так как Rc реализует deref
    let our_str = x.borrow(); // Получаем ссылку на String из RefCell, который нам доступен из Rc напрямую из-за deref
    println!("{}", our_str);
}

fn test_ref_count(){
    // Создаем объект Rc со строкой   
    let a = Rc::new(RefCell::new("a".to_string()));

    // если убрать move, тут и ниже ...
    let a_ref1 = a.clone();
    let f_1 = move || {
        test_ref_func(a_ref1);
    };

    // ... то компилятор всё равно пожалуется, что не удалось переместить значение в замыкание на этой строке
    let a_ref2 = a.clone();
    let f_2 = move || {
        test_ref_func(a_ref2);
    };

    f_1();
    f_2();
    println!("{}", a.borrow());
}

fn test_ref(){
    // Присваиваем ссылку на тип `i32`. 
    // Символ `&` означает, что присваивается ссылка.
    let reference = &4;

    match reference {
        // Если `reference` - это шаблон, который сопоставляется с `&val`,
        // то это приведёт к сравнению:
        // `&i32`
        // `&val`
        // ^ Мы видим, что если отбросить сопоставляемые `&`, 
        // то переменной `val` должно быть присвоено `i32`.
        &val => println!("Получаем значение через деструктуризацию: {:?}", val),
    }

    // Чтобы избежать символа `&`, нужно разыменовывать ссылку до сопоставления.
    match *reference {
        val => println!("Получаем значение через разыменование: {:?}", val),
    }

    // Что если у нас нет ссылки? `reference` была с `&`,
    // потому что правая часть была ссылкой. Но это не ссылка, 
    // потому что правая часть ею не является.
    let _not_a_reference = 3;

    // Rust предоставляет ключевое слово `ref` именно для этой цели. 
    // Оно изменяет присваивание так, что создаётся ссылка для элемента. 
    // Теперь ссылка присвоена.
    let ref _is_a_reference = 3;

    // Соответственно, для определения двух значений без ссылок, 
    // ссылки можно назначить с помощью `ref` и `ref mut`.
    let value = 5;
    let mut mut_value = 6;

    // Используйте ключевое слово `ref` для создания ссылки.
    match value {
        ref r => println!("Получили ссылку на значение: {:?}", r),
    }

    // Используйте `ref mut` аналогичным образом.
    match mut_value {
        ref mut m if (*m == 5) => {
            // Получаем ссылку. Её нужно разыменовать, 
            // прежде чем мы сможем что-то добавить.
            *m += 10;
            println!("Мы добавили 10. `mut_value`: {:?}", m);
        },
        _ => {
            println!("Add failed");
        }
    }    
}

fn test_extend_lifetime(){
    let test_string = "TestTest";

    // Для того, чтобы не аллоцировать бестолку буффер каждый раз
    // Мы можем просто создать пустую переменную, которая как раз будет
    // хранить наши данные
    // Таким образом, уремя жизни удовлетворяется без проблем
    let lowercase_string: String;
    let s: &str = if test_string.chars().all(|symb: char| symb.is_lowercase()){
        test_string
    }else{
        lowercase_string = test_string.to_ascii_lowercase();
        &lowercase_string // Вызывается as ref
    };

    assert_eq!(s, "testtest");

    // Но надо помнить, что эту переменную нельзя никак использовать
    //drop(lowercase_string);
}

fn main() {
    //test_lifetime();
    //test_struct();
    //test_ref_cell();
    //test_ref_count();
    //test_ref();
}