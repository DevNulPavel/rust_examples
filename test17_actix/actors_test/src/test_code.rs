//use actix::fut::*;
//use futures::FutureExt; // Для того, чтобы работали методы у future
use actix::prelude::*;
use futures::prelude::*;

use crate::actors::SummatorActor; // Crate, self или super ???
use crate::actors::SubActor;
use super::value_message::ValuesMessage;
use super::calc_result::CalcResult;


/////////////////////////////////////////////

pub fn test_actor_messages() {
    // TODO: Нужно ли явно создавать систему?
    let _sys = actix::System::new("test");

    // Создаем нашего актора, такой спооб нужен для быстрого создания и запуска потом
    let sum_actor = SummatorActor::default();

    // Закидываем его в систему c получением канала отправки сообщений
    let sum_addr: actix::Addr<SummatorActor> = sum_actor.start();

    // Такой способ создания нужен для создания актора с возможностью доступа к контексту
    // до его создания
    /*let sub_adr = SubActor::create(|_ctx|{
        let sub_actor = SubActor::default();
        sub_actor
    });*/

    // Создаем контекст исполнения для 2х акторов, работающих в пуле
    let sub_adr = actix::SyncArbiter::start(2, ||{
        let sub_actor = SubActor::default();
        sub_actor
    });

    // Отбправляем сообщение, получаем объект с отложенным результатом
    let res1 = sum_addr.send(ValuesMessage::new(10, 5));

    // Можно получить объект, который можно клонировать для отправки сообщений
    // Реципиент - специализированная версия адреса, которая поддерживает только один тип сообщений
    // Может быть использована для случаев, когда нужно отправить сообщение куче разных акторов
    // Как результат - мы можем положить их в один массив
    let sum_recepient = sum_addr.recipient();

    // Для отмены задачи - можно просто уничтожить объект
    let res3 = sum_recepient.send(ValuesMessage::new(1, 2));
    drop(res3);

    // Вариант отправки без необходимости ожидания ответа
    sum_recepient.do_send(ValuesMessage::new(1, 2)).unwrap();
    
    // Можем попытаться отправить
    sum_recepient.try_send(ValuesMessage::new(10, 50)).unwrap();

    // Отправка сообщения с возвратом future
    let res2 = sum_recepient.send(ValuesMessage::new(1, 2));

    let sub_recepient = sub_adr.recipient();

    let all_recepients = vec![sum_recepient, sub_recepient];
    let all_results: Vec<_> = all_recepients
        .iter()
        .map(|recepient|{
            recepient.send(ValuesMessage::new(40, 20))
        })
        .collect();

    let new_arbiter = actix::Arbiter::new();
    new_arbiter.send(futures::future::lazy(move |_|{
        println!("Test");
        // for result in all_results.into_iter(){
        //     println!("{:?}", result.await.unwrap());
        // }
    }));

    // Запускаем задачу в одном единственном потоке арбитра, в EventLoop
    // Arbiter - однопоточный EventLoop
    // actix::Arbiter::spawn(async move {
    // actix::spawn(async move {

    // Короткий вариант, совмещающий запуск и обработку
    actix::run(async move {
        // Можем подождать асинхронно
        actix::clock::delay_for(std::time::Duration::from_millis(2000)).await;

        // Можно создать таймер, который будет срабатывать с определенной периодичностью
        // Можно таким образом создать бесконечный цикл, который что-то делает
        {
            use std::time::Duration;

            let start = tokio::time::Instant::now() + Duration::from_millis(50);
            let mut interval = actix::clock::interval_at(start, Duration::from_millis(10));

            for _ in 0..5 {
                //futures::try_join!(interval.tick());
                interval
                    .tick()
                    .map(|val|{
                        val
                    })
                    .await;
            }
        }

        let result: CalcResult = res1.await.unwrap();
        assert_eq!(result.result, 15);
        assert_eq!(result.operations_count, 1);
        println!("{:?}", result);

        let result: CalcResult = res2.await.unwrap();
        assert_eq!(result.result, 3);
        assert_eq!(result.operations_count, 4);
        println!("{:?}", result);

        /*all_results
            .into_iter()
            .for_each(|result| {
                //assert_eq!(result.result, 3);
                //assert_eq!(result.operations_count, 4);
                println!("{:?}", result.await);
            });*/

        for result in all_results.into_iter(){
            let value = result
                // map преобразует значения одного типа в значения другого
                .map(|result|{
                    if let Ok(valid_value) = result {
                        return valid_value.result;
                    }
                    0_i32
                })
                // Запускает по цепочке новые вычисления, должна возвращать новую future
                .then(|result| {
                    // Новый async блок, который возвращает future
                    let new_future = async move {
                        let temp_result: i32 = result;
                        temp_result
                    };
                    new_future
                })
                .await;
            println!("{:?}", value);
        }

        actix::System::current().stop();
    }).unwrap();

    // Запускаем систему и блокируемся тут
    // sys.run().unwrap();
    
}