use std::sync::Arc;
use std::sync::Mutex;
use core::task::Context;
use core::task::Poll;
use core::task::Waker;
use core::time::Duration;
use core::pin::Pin;
use std::future::Future;
use std::thread;

// Класс общего состояния, который шарится между потоками
struct SharedState {
    completed: bool,    // Флаг завершения

    // Объект Waker нужен для оповещения, что задача готова продолжить свое выполнение и требуется очередная итерация
    waker: Option<Waker>,
}

// Класс, описывающий таймер
pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>,
}

// Реализация интерфейса Future, чтобы можно было работать с await у данного объекта
impl Future for TimerFuture {
    // Вовзращщаемый результат
    type Output = ();

    // Функция, которая вызывается лупером
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Получаем наше состояние с блокировкой
        let mut shared_state = self.shared_state.lock().unwrap();

        // Если на прошлой итерации мы завершили выполнение - можно выходить с успешным результатом
        if shared_state.completed {
            Poll::Ready(()) // Возврат успешного результата
        } else {
            // Сохраняем наш waker для будущего восстановления
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    pub fn new(duration: Duration) -> Self {
        // Создаем пустое новое состояние
        let empty_state = SharedState {
            completed: false,
            waker: None,
        };
        let shared_state = Arc::new(Mutex::new(empty_state));

        // Создаем собщее состояние для передачи в поток
        let thread_shared_state = shared_state.clone();

        // Создаем новый поток с передачей туда всех переменных
        thread::spawn(move || {
            // Засыпаем
            thread::sleep(duration);

            // Снова получаем наше состояние с блокировкой (shared_state - это Mutex, но с доступом к внутренностям)
            let mut shared_state = thread_shared_state.lock().unwrap();

            // Выставляем флаг
            shared_state.completed = true;

            // Получаем Waker и вызываем wake для оповещения о том, что у нас есть результаты
            // TODO: Можно ли вызывать из другого потока?
            if let Some(waker) = shared_state.waker.take() {
                waker.wake()
            }
        });

        TimerFuture { shared_state }
    }
}