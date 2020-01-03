extern crate timer_future;

// Так надо подключать соседние модули
//use crate::folder_test;

use {
	futures::{
		future::{FutureExt, BoxFuture, Future}, // Аналогичено std::future::Future, просто переопределение стандартного типа библиотекой
		task::{ArcWake, waker_ref},
	},
	std::{
		sync::{Arc, Mutex},
		sync::mpsc::{sync_channel, SyncSender, Receiver},
		task::{Context, Poll},
		time::Duration,
	},
    timer_future::TimerFuture,
    crate::folder_test::test_func_1,
    crate::folder_test::test_func_2,
};

////////////////////////////////////////////////////////////////////////////////

// Исполнитель задач, который получает задачи из канала и запускает их
struct Executor {
	ready_queue: Receiver<Arc<Task>>,
}

impl Executor {
	fn run(&self) {
        // Пока мы можем получать из канала новые задачи, получаем их
		while let Ok(task) = self.ready_queue.recv() {
            // Принимаем нашу фьючу, и если она пока не завершена (все еще имеет значение Some),
            // тогда продолжаем ее исполнение
            let mut future_slot = task.future.lock().unwrap();
            if let Some(mut future) = future_slot.take() {
                // Создаем локальную копию Waker из задачи
                let waker = waker_ref(&task);
                // Получаем изменяемый контекст
                let context = &mut Context::from_waker(&*waker);

                // BoxFuture<T> - алиас для Pin<Box<dyn Future<Output = T> + Send + 'static>>
                // Мы можем получить Pin<&mut dyn Future + Send + 'static> с помощью метода Pin::as_mut
                
                // Вызываем у нашей фьючи метод poll с предыдущим контекстом исполнения, чтобы продолжить исполнение
                if let Poll::Pending = future.as_mut().poll(context) {
                    // Если нам нужно продолжать исполнение, то сохраняем задачу на следующий раз
                    *future_slot = Some(future);
                }
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

// Спавнер закидывает новые фьючи в канал тасков Executor
#[derive(Clone)]
struct Spawner {
	task_sender: SyncSender<Arc<Task>>,
}

impl Spawner {
	fn spawn(&self, future: impl Future<Output = ()> + 'static + Send) {
        // Создаем фьючу, собранную в Box
        let future = future.boxed();
        // Создаем задачу
        let task_struct = Task {
			future: Mutex::new(Some(future)), // Сохраняем нашу фьючу
			task_sender: self.task_sender.clone(), // Сохраняем канал
		};
        // Оборачиваем задачу, обеспечивая безопасность многопоточности
        let task = Arc::new(task_struct);
        // Закидываем задачу на исполнение
		self.task_sender.send(task).expect("too many tasks queued");
	}
}

// Реализация трейта ArcWake для возможности прямого вызова wake у Arc
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Заново отправляем нашу задачу в лупер задач
        let cloned = arc_self.clone();
        arc_self.task_sender.send(cloned).expect("too many tasks queued");
    }
}

////////////////////////////////////////////////////////////////////////////////

// Структура, представляющая собой задачу
struct Task {
    // Оборачиваем фьючу в Mutex для возможности блокировки
    future: Mutex<Option<BoxFuture<'static, ()>>>,

    // Канал закидывания задач
    task_sender: SyncSender<Arc<Task>>,
}

////////////////////////////////////////////////////////////////////////////////

fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASKS: usize = 1000;
    
    // Создаем синхронный канал на определенное количество элементов
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASKS);
    
    let executor = Executor { 
        ready_queue 
    };
    let spawner = Spawner { 
        task_sender 
    };
    (executor, spawner)
}

////////////////////////////////////////////////////////////////////////////////

pub fn text_executor_example() {
    // Создаем исполнителя
	let (executor, spawner) = new_executor_and_spawner();

    // Создаем новую задачу
    spawner.spawn(async {
    	println!("Before wait!");
        
        // Создаем новую фьючу, на которой можно ждать
        TimerFuture::new(Duration::new(2, 0)).await;

        println!("After!");
    });

    // Уничтожаем спавнер, чтобы больше нельзя было принимать входящие задачи
    drop(spawner);

    // Run the executor until the task queue is empty.
    // This will print "howdy!", pause, and then print "done!".
    // Выполняем задачи до тех пор, пока
    executor.run();

    test_func_1();
    test_func_2();
}