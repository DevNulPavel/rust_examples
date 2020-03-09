use actix::*;

/////////////////////////////////////////////

// Содержимое нашего сообщения
#[derive(Default)]
struct ValuesMessage{
    x: i32, 
    y: i32
}

impl ValuesMessage{
    fn new(x: i32, y: i32)-> ValuesMessage{
        ValuesMessage{ x, y }
    }
}

// Реализация трейта Message для нашего сообщения
impl actix::Message for ValuesMessage {
    // описываем тип возвращаемого значения на сообщение
    //type Result = Option<CalcResult>;
    type Result = CalcResult;
}

/////////////////////////////////////////////

#[derive(Default, Debug)]
struct CalcResult{
    result: i32,
    operations_count: u32
}

impl CalcResult{
    fn new(result: i32, operations_count: u32)-> CalcResult{
        CalcResult{ result, operations_count }
    }
}

// Для того, чтобы не использовать Option или Result в описании типа Result у сообщения
// и у актора - мы реализуем данный трейт для результата сообщения
// Для Option/Result - уже реализовано в самой библиотеке
impl<A, M> actix::dev::MessageResponse<A, M> for CalcResult
where
    A: actix::Actor,
    M: actix::Message<Result = CalcResult>
{
    fn handle<R: actix::dev::ResponseChannel<M>>(self, _: &mut A::Context, tx: Option<R>) {
        if let Some(tx) = tx {
            tx.send(self);
        }
    }
}

/////////////////////////////////////////////

// Описание нашего актора
#[derive(Default)]
struct SummatorActor{
    messages_processed: u32 // Контекст актора
}

// Реализация трейта актора
impl actix::Actor for SummatorActor {
    // Описываем контекст актора
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        // Можно ограничить размер очереди
        // https://actix.rs/book/actix/sec-4-context.html
        ctx.set_mailbox_capacity(1);
        println!("Sum actor started, state: {:?}", ctx.state());
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Sum actor stopped, state: {:?}", ctx.state());
    }
}

// Описываем обработку сообщения SumMessage для нашего актора
impl actix::Handler<ValuesMessage> for SummatorActor {
    //type Result = Option<SumResult>;   // Описываем возвращаемое значение для актора
    type Result = CalcResult;   // Описываем возвращаемое значение для актора, реализовали MessageResponse

    // Обработчик поступившего сообщения для актора
    fn handle(&mut self, msg: ValuesMessage, ctx: &mut actix::Context<Self>) -> Self::Result {
        if ctx.connected(){
            println!("Context is connected");
        }
        println!("Actor state: {:?}", ctx.state());

        // Обработка происходит из только одного потока, синхронизация не нужна
        self.messages_processed += 1;

        // Вычисляем
        let sum_value: i32 = msg.x + msg.y;
        
        // Результат
        Self::Result::new(sum_value, self.messages_processed)
    }
}

/////////////////////////////////////////////

// Описание нашего актора
#[derive(Default)]
struct SubActor{
    messages_processed: u32 // Контекст актора
}

// Реализация трейта актора
impl actix::Actor for SubActor {
    // Описываем контекст актора
    type Context = actix::Context<Self>;
}

// Описываем обработку сообщения SumMessage для нашего актора
impl actix::Handler<ValuesMessage> for SubActor {
    //type Result = Option<SumResult>;   // Описываем возвращаемое значение для актора
    type Result = CalcResult;   // Описываем возвращаемое значение для актора, реализовали MessageResponse

    // Обработчик поступившего сообщения для актора
    fn handle(&mut self, msg: ValuesMessage, _ctx: &mut actix::Context<Self>) -> Self::Result {
        // Обработка происходит из только одного потока, синхронизация не нужна
        self.messages_processed += 1;

        // Вычисляем
        let sub_value: i32 = msg.x - msg.y;
        
        // Результат
        Self::Result::new(sub_value, self.messages_processed)
    }
}

/////////////////////////////////////////////

pub fn test_actor_messages() {
    let sys = actix::System::new("test");

    // Создаем нашего актора, такой спооб нужен для быстрого создания и запуска потом
    let sum_actor = SummatorActor::default();

    // Закидываем его в систему c получением канала отправки сообщений
    let sum_addr: actix::Addr<SummatorActor> = sum_actor.start();

    // Такой способ создания нужен для создания актора с возможностью доступа к контексту
    // до его создания
    let sub_adr = SubActor::create(|_ctx|{
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

    actix::Arbiter::spawn(async move {
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
            println!("{:?}", result.await.unwrap());
        }

        actix::System::current().stop();
    });

    sys.run().unwrap();
}