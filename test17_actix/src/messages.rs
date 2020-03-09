use actix::*;


// Содержимое нашего сообщения
#[derive(Default)]
struct SumMessage{
    x: usize, 
    y: usize
}

impl SumMessage{
    fn new(x: usize, y: usize)-> SumMessage{
        SumMessage{ x, y }
    }
}

// Реализация трейта Message для нашего сообщения
impl actix::Message for SumMessage {
    // описываем тип возвращаемого значения на сообщение
    type Result = Result<SumResult, String>;
}

/////////////////////////////////////////////

#[derive(Default, Debug)]
struct SumResult{
    result: usize,
    sum_count: u32
}

impl SumResult{
    fn new(result: usize, sum_count: u32)-> SumResult{
        SumResult{ result, sum_count }
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
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Sum actor started, state: {:?}", ctx.state());
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Sum actor stopped, state: {:?}", ctx.state());
    }
}

// Описываем обработку сообщения SumMessage для нашего актора
impl actix::Handler<SumMessage> for SummatorActor {
    type Result = Result<SumResult, String>;   // Описываем возвращаемое значение для актора

    // Обработчик поступившего сообщения для актора
    fn handle(&mut self, msg: SumMessage, ctx: &mut actix::Context<Self>) -> Self::Result {
        if ctx.connected(){
            println!("Context is connected");
        }
        println!("Actor state: {:?}", ctx.state());

        // Обработка происходит из только одного потока, синхронизация не нужна
        self.messages_processed += 1;

        // Вычисляем
        let sum_value: usize = msg.x + msg.y;
        
        // Результат
        Ok(SumResult::new(sum_value, self.messages_processed))
    }
}

/////////////////////////////////////////////

pub fn test_actor_messages() {
    let sys = actix::System::new("test");

    // Создаем нашего актора
    //let actor = SummatorActor::default();

    // Закидываем его в систему c получением канала отправки сообщений
    //let addr: actix::Addr<SummatorActor> = actor.start();

    let addr = SummatorActor::create(|_ctx|{
        let summator = SummatorActor::default();

        summator
    });

    // Отбправляем сообщение, получаем объект с отложенным результатом
    let res1 = addr.send(SumMessage::new(10, 5));

    // Можно получить объект, который можно клонировать для отправки сообщений
    let recepient = addr.recipient();

    // Отправка сообщения с возвратом future
    let res2 = recepient.send(SumMessage::new(1, 2));

    // Для отмены задачи - можно просто уничтожить объект
    let res3 = recepient.send(SumMessage::new(1, 2));
    drop(res3);

    // Вариант отправки без необходимости ожидания ответа
    recepient.do_send(SumMessage::new(1, 2)).unwrap();
    
    actix::Arbiter::spawn(async {
        let result = res1.await.unwrap().unwrap();
        println!("{:?}", result);

        let result = res2.await.unwrap().unwrap();
        println!("{:?}", result);

        actix::System::current().stop();
    });

    sys.run().unwrap();
}