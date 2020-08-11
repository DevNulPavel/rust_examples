use actix::{
    prelude::{
        *
    }
};
use futures::{
    // prelude::{
    //     *
    // },
    FutureExt
};
use super::{
    message::{
        Ping
    },
    actor::{
        PingPongActor
    }
};

async fn ping_pong_logic(){
    // Создаем нашего актора, такой спооб нужен для быстрого создания и запуска потом
    let sum_actor = PingPongActor{};

    // Запускаем и получаем адрес
    let addr = sum_actor.start();

    // Отправляем сообщение Ping
    // send() возвращает объект Future , который резолвится в результат
    let result_future: Request<PingPongActor, Ping> = addr.send(Ping{});

    // Ждем результат
    let res = result_future.await;

    // Выводим
    match res {
        Ok(result) => {
            println!("Got result: {:?}", result)
        },
        Err(err) => {
            println!("Got error: {}", err)
        },
    }

    // Останавливаем систему
    System::current().stop();
}

pub fn test_ping_pong() {
    let sys = System::new("example");

    // Закидываем future в реактор
    Arbiter::spawn(ping_pong_logic());

    sys.run().unwrap();
}