#![allow(dead_code, unused_variables)]

use std::future::Future;
use futures::future;

// Данная функция async эквивалентная той, что ниже
async fn foo(x: &u8) -> u8 { 
    *x 
}

// Так выглядит обычная функция, async оборачивает выходное значение в трейт Future
fn foo_expanded<'a>(x: &'a u8) -> impl Future<Output = u8> + 'a {
    async move { 
        *x 
    }
}

async fn borrow_x(x: &u8) -> u8{
    *x
}

fn async_lifetime() -> impl Future<Output = u8> {
    let x = 5;

    // Тут будет ошибка "`x` does not live long enough", так как корутина может исполняться после выхода из данной функции
    //borrow_x(&x)
    
    // Чтобы исправить текущую проблему, нужно хранить переменную в контексте асинхронной функции
    async {
        let x = 5;
        borrow_x(&x).await
    }
}

async fn blocks() {
    // Несколько асинхронных блоков могут получить доступ к одной локальной переменной,
    // так как они исполняются в одном пространстве
    let my_string = "foo".to_string();

    let future_one = async {
        println!("{}", my_string);
    };

    let future_two = async {
        println!("{}", my_string);
    };

    // Дожидаемся завершения
    let ((), ()) = future::join(future_one, future_two).await;
}

fn move_block() -> impl Future<Output = ()> {
    // Так как переменная перемещается, то данная переменная больше не будет доступна в блоках
    let my_string = "foo".to_string();
    async move {
        println!("{}", my_string);
    }
}

pub fn async_await_test (){

}