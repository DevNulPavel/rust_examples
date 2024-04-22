use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::Mutex;
use mpsc::Receiver;
use mpsc::SyncSender;
// use mpsc::Sender;
// use mpsc::SendError;


// Описываем трейт вызова метода
trait FnBox {
    fn call_box(self: Box<Self>);
}

// Для всех FnOnce реализуем интерфейс call_box
impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

// Сообщение должно представлять из себя функцию-задачу, способную перемещаться между потоками
// и хранящуюся в статической области видимости
type Job = Box<dyn FnBox + Send + 'static>;

// Описываем enum сообщений
enum Message {
    NewJob(Job),
    Terminate,
}

/////////////////////////////////////////////////////////////////////////////////////

struct Worker {
    id: usize,
    thread: Option<JoinHandle<()>>,
}

type WorkerReceiver = Arc<Mutex<Receiver<Message>>>;

impl Worker {
    fn new(id: usize, receiver: WorkerReceiver) -> Worker {
        let thread = thread::spawn(move ||{
            loop {
                // Блокировка живет только на время получения сообщения
                let message = if let Ok(lock) = receiver.lock() {
                    match lock.recv() {
                        Ok(message) => {
                            message
                        },
                        Err(_)=>{
                            println!("Channel receive failed");
                            break;
                        }
                    }
                }else{
                    print!("Break failed");
                    break;
                };
                
                // Обрабатываем поступившее сообщение, смотрим что оно из себя представляет
                match message {
                    // Либо выполняем работу
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);
                        job.call_box();
                    },
                    // Либо прерываем работу потока
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);
                        break;
                    },
                }
            }
        });
        
        Worker {
            id,
            thread: Some(thread),
        }
    }
}

/////////////////////////////////////////////////////////////////////////////////////

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: SyncSender<Message>,
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize, backpressure_size: usize) -> ThreadPool {
        assert!(size > 0);
        
        // Создаем канал для работы
        let (sender, receiver) = mpsc::sync_channel(backpressure_size);
        
        // Так как из канала может принимать тольк один поток за раз, тогда просто
        // оборачаваем получаетеля канала в блокировку
        let receiver: WorkerReceiver = Arc::new(Mutex::new(receiver));
        
        // Создаем вектор воркеров
        let mut workers = Vec::with_capacity(size);
        
        // Добавляем воркеров в вектор
        for id in 0..size {
            workers.push(Worker::new(id, receiver.clone()));
        }
        
        // Возвращаем экземпляр
        ThreadPool {
            workers,
            sender,
        }
    }
    
    // Вкидывание задачи на исполнение
    pub fn execute<F>(&self, f: F) -> Result<(), String>
        where 
            // FnOnce - функция, которая может быть вызвана лишь раз, функция будет вызвана внутри пула
            // Send - это трейт компилятора, который обозначает, что значение можно мувить из одного потока в другой
            // 'static - время жизни статичное у данной функции
            F: FnOnce() + Send + 'static 
    {

        // Оборачиваем функцию в unique_ptr
        let job = Box::new(f);
        
        // Отсылаем новое сообщение
        if let Err(err) = self.sender.send(Message::NewJob(job)){
            return Err(format!("Thread pool send message failed: {}", err));
        }

        return Ok(());
    }
}

// Для пула потоков реализуем интерфейс деструктора, чтобы дождаться завершения потоков
impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!("Sending terminate message to all workers.");
        
        // Отправляем всем потокам сообщение о необходимости завершения
        for _ in &mut self.workers {
            if self.sender.send(Message::Terminate).is_err() {
                println!("Failed to send terminate in thread pool");
            }
        }
        
        println!("Shutting down all workers.");
        
        // Цепляемся к каждому потоку для ожидания завешршения
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);
            
            // take нужен, чтобы переместить владение в объект Some(thread)
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

