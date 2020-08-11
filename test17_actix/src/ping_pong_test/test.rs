use actix::{
    prelude::{
        *
    }
};
/*use futures::{
    prelude::{
        *
    },
};*/
use super::{
    ping_message::{
        Ping
    },
    ping_subscribe_message::{
        PingSubscribe
    },
    ping_actor::{
        PingActor
    },
    counter::{
        CounterActor
    }
};

async fn ping_pong_logic(){
    // Актор подсчета увеличений
    // Создаем реципиента из адреса, нужен для отправки конкретного типа сообщения многим акторам
    let counter_actor_recepient_1 = CounterActor::new("Counter 1").start().recipient();
    let counter_actor_recepient_2 = CounterActor::new("Counter 2").start().recipient();

    // Создаем нашего актора, такой спооб нужен для быстрого создания и запуска потом
    let ping_actor_addr = PingActor::new().start();

    // Подписываем обсервера на увеличение счетчика
    ping_actor_addr.do_send(PingSubscribe::new(counter_actor_recepient_1));
    ping_actor_addr.do_send(PingSubscribe::new(counter_actor_recepient_2));

    // Отправляем сообщение без ожидания результата
    ping_actor_addr.do_send(Ping{});

    // Отправляем сообщение Ping
    // send() возвращает объект Future , который резолвится в результат
    let result_future: Request<PingActor, Ping> = ping_actor_addr.send(Ping{});

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
    //System::current().stop();
}

pub fn test_ping_pong() {
    /*// Создаем систему, она должна жить достаточно долго
    let sys = System::new("ping_pong_system");

    // Закидываем future в реактор
    Arbiter::spawn(ping_pong_logic());

    // Запускаем систему
    sys.run()
        .unwrap();*/

    actix::run(ping_pong_logic()).unwrap();
}