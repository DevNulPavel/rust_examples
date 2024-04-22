use lapin::{
    options::{
        QueueDeclareOptions,
        BasicConsumeOptions,
        BasicAckOptions,
        BasicPublishOptions,
        BasicQosOptions,
        ExchangeDeclareOptions,
        QueueBindOptions
    },
    types::{
        FieldTable
    },
    BasicProperties,
    Queue,
    Channel,
    Connection, 
    ConnectionProperties,
    ExchangeKind
};
use tokio_amqp::{
    LapinTokioExt
};
use futures::{
    StreamExt,
    TryFutureExt
};
use tracing::{
    info,
    error,
    info_span,
    instrument
};
use tracing_futures::{
    Instrument
};
use crate::{
    ok_or,
    unwrap_or_else
};

#[instrument(skip(rabbit_chan, queue_name))]
async fn consume_data(rabbit_chan: Channel, queue_name: &str) {
    // Создаем получателя для наших данных
    let mut consumer = rabbit_chan
        .basic_consume(queue_name, 
                       "", 
                       BasicConsumeOptions{
                           ..Default::default()
                       },
                       FieldTable::default())
        .await
        .expect("Consumer create failed");
    info!("Consumer created");

    while let Some(data) = consumer.next().await{
        // Получаем зачем-то канал и собственно нашу доставку
        let (_, delivery) = unwrap_or_else!(data, err => {
            error!(%err, "Rabbit consume error");
            continue;
        });

        // Разворачиваем в обычный текст
        let text = unwrap_or_else!(std::str::from_utf8(delivery.data.as_ref()), err => {
            error!(?err, "UTF-8 parse failed");
            continue;
        });

        // Выводим содержимое
        info!(?text, "Rabbit data received");

        // Спим какое-то время
        tokio::time::sleep(std::time::Duration::from_millis(1000))
            .await;

        // Выполняем подтверждение обработки наших данных
        ok_or!(delivery.ack(BasicAckOptions::default()).await, err => {
            error!(%err, "Ack delivery failed");
        });
    }
}

#[instrument(skip(channel, exchange_name))]
async fn produce_data(channel: Channel, exchange_name: &str) {
    loop {
        tokio::time::sleep(std::time::Duration::from_millis(500))
            .await;

        use rand::Rng;
        let val = rand::thread_rng()
            .gen_range(1..4);
        let route_name = format!("route_{}", val);

        // Если не указать обменник, то попадает в стандартную очередь, 
        // а роутинг идет просто по имени ключа роутинга
        // Подтверждение нужно для информации, что долговременное сообщение сохранилось на диск
        let confirm = channel
            .basic_publish(exchange_name, 
                           route_name.as_str(), 
                           BasicPublishOptions{
                                ..Default::default()
                           }, 
                           b"My message data".to_vec(), 
                           BasicProperties::default())
            .inspect_ok(|_publish|{
                // info!(?publish, "Publish value");
            })
            .inspect_err(|err| {
                error!(%err, "Publish error");
            })                                            
            .and_then(|confirm_awaiter| async {
                confirm_awaiter.await
            })
            .await;

        // Разворачиваем наше подтверждение
        let _confirm = unwrap_or_else!(confirm, err => {
            error!(%err, "Confirmation error");
            continue;
        });

        // info!(?confirm, "Publish value");
    }
}

#[instrument(skip(rabbit_chan, exchange_name, routing_key))]
async fn create_and_bind_queue(rabbit_chan: &Channel, 
                               exchange_name: &str, 
                               routing_key: &str) -> Queue {
    let queue = rabbit_chan
        .queue_declare("", 
                        QueueDeclareOptions{
                            durable: false,
                            auto_delete: true,
                            exclusive: false,
                            ..Default::default()
                        }, 
                        FieldTable::default())
        .await
        .expect("Queue create failed");
    info!(?queue, "Queue info");

    // Привязываем нашу очередь к обменнику
    rabbit_chan
        .queue_bind(queue.name().as_str(), 
                    exchange_name, 
                    routing_key,
                    QueueBindOptions::default(), 
                    FieldTable::default())
        .await
        .expect("Queue exchange bind error");

    info!(?queue, "Queue create and bind success");

    queue
}

#[instrument]
pub async fn routing_example() {

    // Создаем соединение с сервером rabbit
    let rabbit_connection_properties = ConnectionProperties::default()
        .with_tokio();
    let rabbit_conn = Connection::connect("amqp://guest:guest@127.0.0.1:5672", rabbit_connection_properties)
        .await
        .expect("Rabbit connection failed");
    info!("Rabbit connection established");

    // Создаем канал общения с rabbit
    let rabbit_chan = rabbit_conn
        .create_channel()
        .await
        .expect("Channel create failed");

    // Имя нашей рабочей очереди
    let exchange_name = "my_exchange_name";

    // Настраиваем наш обменник на раздачу всем пользователям
    rabbit_chan
        .exchange_declare(exchange_name, 
                          ExchangeKind::Direct, // Роутинг будет происходить на основании передаваемого маршрута 
                          ExchangeDeclareOptions{
                              auto_delete: true,
                              durable: true,
                              ..Default::default()
                          }, 
                          FieldTable::default())
        .await
        .expect("Exchange declare failed");

    // Создаем очереди c пустым именем, тем самым давая возможность сгенерировать имя очереди
    // Одновременно привязываем данные очереди к обменнику
    let queue1 = create_and_bind_queue(&rabbit_chan, exchange_name, "route_1")
        .await;
    let queue2 = create_and_bind_queue(&rabbit_chan, exchange_name, "route_2")
        .await;
    let queue3 = create_and_bind_queue(&rabbit_chan, exchange_name, "route_3")
        .await;

    // Коворим, чтобы rabbit не давал задачу новому воркеру до тех пор, пока
    // тот не обработал прошлое сообщение
    rabbit_chan
            .basic_qos(1, BasicQosOptions{
                ..Default::default()
            })
            .await
            .expect("QOS setup failed");

    // Producers and consumers
    tokio::join!(produce_data(rabbit_chan.clone(), exchange_name), 
                 consume_data(rabbit_chan.clone(), queue1.name().as_str())
                     .instrument(info_span!("consumer_1")), 
                 consume_data(rabbit_chan.clone(), queue2.name().as_str())
                     .instrument(info_span!("consumer_2")),
                 consume_data(rabbit_chan, queue3.name().as_str())
                     .instrument(info_span!("consumer_3")));
    info!("Consumers and producers complete");
}