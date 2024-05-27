#![warn(clippy::all)]

use std::thread;
use futures::prelude::*;
use futures::future::Future;
use futures::future::FutureExt;
use futures::executor;
use futures::executor::ThreadPool;
use futures::channel::mpsc;
use futures::channel::oneshot;


fn test_pool(){
    let pool = ThreadPool::new().expect("Failed to build pool");
    let (tx, rx) = mpsc::unbounded::<i32>();

    // Создаем фьючу описывая асинхронный блок. Блок не выполняется пока мы не отдадим его исполнителю
    let fut_values = async {
        // Создаем новый асинхронный блок внутри имеющегося
        let fut_tx_result = async move {
            (0..100).for_each(|v| {
                tx.unbounded_send(v).expect("Failed to send");
            })
        };

        // Запускаем задачу в пуле потоков
        pool.spawn_ok(fut_tx_result);

        // Получаем результаты и собираем их в кучу
        let fut_values = rx
            .map(|v| v * 2)
            .collect();

        // Дожидаемся результата
        fut_values.await
    };

    // Блокируемся на текущем потоке, вызываем poll у нашей задачи до тех пор, пока не завершится
    let values: Vec<i32> = executor::block_on(fut_values);

    println!("Values={:?}", values);    
}

fn test_channels(){
    // Создаем одноразовый канал
    let (sender, receiver) = oneshot::channel::<i32>();

    thread::spawn(move || {
        sender.send(3).unwrap(); 
    });

    executor::block_on(async move {
        let val = receiver.await.unwrap();
        println!("got: {:?}", val);
    });
}

fn main() {
    test_pool();
    test_channels();
}
