// https://doc.rust-lang.ru/stable/rust-by-example/macros/dry.html

use std::ops::{Add, Mul, Sub};

// Этот простой макрос называется say_hello.
macro_rules! say_hello {
    // () указывает, что макрос не принимает аргументов.
    () => (
        // Макрос будет раскрываться с содержимым этого блока.
        println!("Hello!");
    )
}

// block
// expr - используют для обозначения выражений
// ident - используют для обозначения имени переменной/функции
// item
// literal - используется для литеральных констант
// pat (образец)
// path
// stmt (единственный оператор)
// tt (единственное дерево лексем)
// ty (тип)
// vis (спецификатор видимости)

macro_rules! create_function {
    // Этот макрос принимает аргумент идентификатора ident и
    // создаёт функцию с именем $func_name.
    // Идентификатор ident используют для обозначения имени переменной/функции.
    ($func_name:ident) => (
        fn $func_name() {
            // Макрос stringify! преобразует ident в строку.
            println!("Вызвана функция {:?}()", stringify!($func_name))
        }
    )
}

// Создадим функции с именами foo и bar используя макрос, указанный выше.
create_function!(foo);
create_function!(bar);

macro_rules! print_result {
    // Этот макрос принимает выражение типа expr и напечатает его как строку вместе с результатом.
    // Указатель expr используют для обозначения выражений.
    ($expression:expr) => (
        // stringify! преобразует выражение в строку *без изменений*
        println!("{:?} = {:?}", stringify!($expression), $expression);
    )
}

// test! будет сравнивать $left и $right
// по разному, в зависимости от того, как вы объявите их:
macro_rules! test {
    // Не нужно разделять аргументы запятой.
    // Можно использовать любой шаблон!
    ($left:expr; and $right:expr) => (
        println!("{:?} и {:?} это {:?}", stringify!($left), stringify!($right), $left && $right)
    );
    // ^ каждый блок должен заканчиваться точкой с запятой.
    ($left:expr; or $right:expr) => (
        println!("{:?} или {:?} это {:?}", stringify!($left), stringify!($right), $left || $right)
    );
}

// min! посчитает минимальное число аргументов.
macro_rules! find_min {
    // Простой вариант:
    ($x:expr) => ($x);
    // $x следует хотя бы одному $y,
    ($x:expr, $($y:expr),+) => (
        // Вызовем find_min! на конце $y
        std::cmp::min($x, find_min!($($y),+))
    )
}

macro_rules! assert_equal_len {
    // Указатель `tt` (единственное дерево лексем) используют для
    // операторов и лексем.
    ($a:expr, $b:expr, $func:ident, $op:tt) => (
        assert!($a.len() == $b.len(),
                "{:?}: несоответствие размеров: {:?} {:?} {:?}",
                stringify!($func),
                ($a.len(),),
                stringify!($op),
                ($b.len(),));
    )
}

macro_rules! op {
    ($func:ident, $bound:ident, $op:tt, $method:ident) => (
        fn $func<T: $bound<T, Output=T> + Copy>(xs: &mut Vec<T>, ys: &Vec<T>) {
            assert_equal_len!(xs, ys, $func, $op);

            for (x, y) in xs.iter_mut().zip(ys.iter()) {
                *x = $bound::$method(*x, *y);
                // *x = x.$method(*y);
            }
        }
    )
}

// Реализуем функции `add_assign`, `mul_assign`, и `sub_assign`.
op!(add_assign, Add, +=, add);
op!(mul_assign, Mul, *=, mul);
op!(sub_assign, Sub, -=, sub);

#[allow(unused_macros)]
macro_rules! calculate {
    (eval $e:expr) => {{
        {
            let val: usize = $e; // Заставим быть переменную целым числом.
            println!("{} = {}", stringify!{$e}, val);
        }
    }};
}

macro_rules! calculate {
    // Шаблон для единичного `eval`
    (eval $e:expr) => {{
        {
            let val: usize = $e; // Заставим быть переменную целым числом.
            println!("{} = {}", stringify!{$e}, val);
        }
    }};

    // Рекурсивно декомпозируем несколько `eval`
    (eval $e:expr, $(eval $es:expr),+) => {{
        calculate! { eval $e }
        calculate! { $(eval $es),+ }
    }};
}

fn main() {
    // Этот вызов будет раскрыт в код println!("Hello");
    say_hello!();

    foo();
    bar();

    print_result!(1u32 + 1);

    // Напомним, что блоки тоже являются выражениями!
    print_result!({
        let x = 1u32;

        x * x + 2 * x - 1
    });

    test!(1i32 + 1 == 2i32; and 2i32 * 2 == 4i32);
    test!(true; or false);

    println!("{}", find_min!(1u32));
    println!("{}", find_min!(1u32 + 2 , 2u32));
    println!("{}", find_min!(5u32, 2u32 * 3, 4u32));

    calculate! {
        eval 1 + 2 // хе-хе, `eval` _не_ ключевое слово Rust!
    }

    calculate! {
        eval (1 + 2) * (3 / 4)
    }

    calculate! { // Смотри, мама! Вариативный `calculate!`!
        eval 1 + 2,
        eval 3 + 4,
        eval (2 * 3) + 1
    }
}

mod test {
    use std::iter;
    macro_rules! test {
        ($func: ident, $x:expr, $y:expr, $z:expr) => {
            #[test]
            fn $func() {
                for size in 0usize..10 {
                    let mut x: Vec<_> = iter::repeat($x).take(size).collect();
                    let y: Vec<_> = iter::repeat($y).take(size).collect();
                    let z: Vec<_> = iter::repeat($z).take(size).collect();

                    super::$func(&mut x, &y);

                    assert_eq!(x, z);
                }
            }
        }
    }

    // Протестируем `add_assign`, `mul_assign` и `sub_assign`
    test!(add_assign, 1u32, 2u32, 3u32);
    test!(mul_assign, 2u32, 3u32, 6u32);
    test!(sub_assign, 3u32, 2u32, 1u32);
}