use crate::{
    constants::KB,
    helpers::{get_payload, ExecutionResult},
};
use iceoryx2::{
    port::{publisher::Publisher, subscriber::Subscriber},
    prelude::*,
};
use std::{
    process::{Child, Command},
    thread::sleep,
    time::{Duration, Instant},
};

////////////////////////////////////////////////////////////////////////////////

/// Обертка для тестов
pub struct IceoryxWrapper {
    /// Отправитель и получатель
    pub publisher: Publisher<ipc::Service, [u8], ()>,

    /// Получатель
    pub subscriber: Subscriber<ipc::Service, [u8], ()>,
}

impl IceoryxWrapper {
    pub fn new(is_producer: bool, data_size: usize) -> IceoryxWrapper {
        // Билдер отдельной ноды
        let node = NodeBuilder::new().create::<ipc::Service>().unwrap();

        // Имя запроса
        let request_name = ServiceName::new("Request").unwrap();

        // Создаем сервис запроса
        let request_service = node
            .service_builder(&request_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();

        // Имя сервиса ответа
        let response_name = ServiceName::new("Respose").unwrap();

        // Создаем сервис для ответов с той же нодой
        let response_service = node
            .service_builder(&response_name)
            .publish_subscribe::<[u8]>()
            .open_or_create()
            .unwrap();

        // Отправитель и получатель
        // Смотря на какой мы стороне, используем разные каналы
        let (publisher, subscriber) = if is_producer {
            (
                // Создаем сервис отправки
                request_service
                    .publisher_builder()
                    .max_slice_len(data_size)
                    .create()
                    .unwrap(),
                // Создаем сервис подписки для получения
                response_service.subscriber_builder().create().unwrap(),
            )
        } else {
            (
                // Если мы получатель, тогда для ответов создаем сервис отправки
                response_service
                    .publisher_builder()
                    .max_slice_len(data_size)
                    .create()
                    .unwrap(),
                // А здесь уже сервис для получения
                request_service.subscriber_builder().create().unwrap(),
            )
        };

        IceoryxWrapper {
            publisher,
            subscriber,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct IceoryxRunner {
    child_proc: Option<Child>,
    wrapper: IceoryxWrapper,
    request_data: Vec<u8>,
    response_data: Vec<u8>,
    data_size: usize,
}

impl IceoryxRunner {
    pub fn new(start_child: bool, data_size: usize) -> IceoryxRunner {
        // Создаем отправителя
        let wrapper = IceoryxWrapper::new(true, data_size);

        // Получаем путь к сполняемому файлику
        let exe = crate::helpers::executable_path("iceoryx_consumer");

        // Запускаем дочерний процесс получателя
        let child_proc = if start_child {
            Some(
                Command::new(exe)
                    .args(&[data_size.to_string()])
                    .spawn()
                    .unwrap(),
            )
        } else {
            None
        };

        // Awkward sleep again to wait for consumer to be ready
        sleep(Duration::from_secs(2));

        // Генерируем тестовые данные
        let (request_data, response_data) = get_payload(data_size);

        Self {
            child_proc,
            wrapper,
            request_data,
            response_data,
            data_size,
        }
    }

    /// Запуск теста
    pub fn run(&mut self, n: usize, print: bool) {
        let start = Instant::now();
        for _ in 0..n {
            // "Аллоциюуем" буфер для отправки данных
            self.wrapper
                .publisher
                .loan_slice_uninit(self.data_size)
                .unwrap()
                // Теперь буда можем положить тестовые данные
                .write_from_slice(self.request_data.as_slice())
                // И отправить
                .send()
                .unwrap();

            // Waiting for response
            loop {
                // Ждем какого-то ответа
                if let Some(recv_payload) = self.wrapper.subscriber.receive().unwrap() {
                    // Дополнительно отвалидируем еще данные на равенство
                    #[cfg(debug_assertions)]
                    if recv_payload.ne(&self.response_data) {
                        panic!("Sent request didn't get response")
                    }

                    break;
                }
            }
        }

        // Замер времени
        if print {
            let elapsed = start.elapsed();
            let res =
                ExecutionResult::new(format!("Iceoryx - {}KB", self.data_size / KB), elapsed, n);
            res.print_info();
        }
    }
}

impl Drop for IceoryxRunner {
    fn drop(&mut self) {
        // Если у нас здесь есть дочерний процесс, то пробуем ег завершить
        if let Some(ref mut c) = self.child_proc {
            c.kill().unwrap();
        }
    }
}
