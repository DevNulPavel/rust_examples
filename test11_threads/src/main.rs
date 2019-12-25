// Таким образом можно указать для всего файла (#!) отключение неиспользуемых переменных
#![allow(unused_variables, unused_mut, dead_code)]

use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::sync::{Mutex, Arc};

fn test_variables_move(){
    let v = vec![1, 2, 3];

    // Так как у нас прописано move - то владение переменной v
    // переместится в поток
    let thread_handle = thread::spawn(move || {
        for i in 1..10 {
            println!("hi number {} from the spawned thread!", i);
        }
        println!("Here's a vector: {:?}", v);
    });

    for i in 1..5 {
        println!("hi number {} from the main thread!", i);
    }

    let join_result = thread_handle.join();
    let thread_result = join_result.expect("Join failed"); // При возникновении ошибки - напишется ошибка
    //let thread_result = join_result.unwrap(); // Выдает результат, иначе паникует
    /*match join_result {
        Ok(_) => println!("Joined"),
        Err(error) => println!("Join failed with error {:?}", error)
    }*/
    println!("Thread result: {:?}", thread_result);
}

fn test_channel_1(){
    // Создаем канал из двух переменных
    let (tx, rx) = mpsc::channel();

    // Создаем поток 1
    let tx1 = tx.clone(); // Для каждого потока создадим свою версию передатчика
    thread::spawn(move || {
        let val = String::from("hi");
        tx1.send(val).unwrap();
        //println!("val is {}", val); // Данный код невалидный, так как мы отправили данные в канал

        let vals = vec![
            String::from("hi"),
            String::from("from"),
            String::from("the"),
            String::from("thread"),
        ];

        for val in vals {
            tx1.send(val).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Создаем поток 2
    let tx2 = tx.clone();  // Для каждого потока создадим свою версию передатчика
    thread::spawn(move || {
        let val = String::from("hi");
        tx2.send(val).unwrap();
        //println!("val is {}", val); // Данный код невалидный, так как мы отправили данные в канал

        let vals = vec![
            String::from("hi_"),
            String::from("from_"),
            String::from("the_"),
            String::from("thread_"),
        ];

        for val in vals {
            tx2.send(val).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    // Ждем значений из потока
    let received = rx.recv().unwrap();
    println!("Got: {}", received);

    for received in rx {
        println!("Got: {}", received);
    }
}

fn test_channel_2(){
    use mpsc::channel;
    use mpsc::sync_channel;
    use mpsc::Sender;
    use mpsc::Receiver;
    use mpsc::SyncSender;
    //use std::cell::RefCell;
    //use std::sync::Arc;

    // Создаем канал для передачи данных в поток, канал неблокирующий и без ограничения размера
    let (to_thread_tx, to_thread_rx): (Sender<String>, Receiver<String>) = channel();

    // Создаем канал для получения данных из потока, канал блокирующий c буффером в 0 элементов,
    // то есть пока получатель не прочитает - не идем дальше
    let (from_thread_tx, from_thread_rx): (SyncSender<String>, Receiver<String>) = sync_channel(0);

    // Создаем поток
    let thread_handle = thread::spawn(move || {
        println!("THREAD: started");
        // Пока получаем из потока данные - крутимся в цикле
        while let Ok(ref val) = to_thread_rx.recv() {
            println!("THREAD: data received");

            // Создаем строку в верхнем регистре
            let new_text = format!("Hello form thread with {}", val.to_uppercase());

            // Спим
            thread::sleep(Duration::from_secs(1));

            // Отправляем, если не смогли отправить, пробуем несколько раз еще, иначе - выходим
            println!("THREAD: try to send");
            if let Err(_) = from_thread_tx.send(new_text) {
                println!("THREAD: send failed");
                return; // Выходим при ошибке
            }
        }

        println!("THREAD: exit");
    });

    println!("MAIN: try to send data to thread");

    // Отправляем в поток
    for i in 0..5{
        let text = format!("Text {}", i);
        println!("MAIN: try to send {}", text.as_str());
        if let Ok(_) = to_thread_tx.send(text) {
            println!("MAIN: send success");
        }else{
            println!("MAIN: failed to send");
        }
    }

    // Накидали все данные, больше не будем отсылать
    drop(to_thread_tx);

    // Получаем
    while let Ok(text) = from_thread_rx.recv() {
        println!("MAIN: received-> {}", text);
    }
    println!("MAIN: receive complete");

    // Ждем поток
    thread_handle.join().unwrap();

    println!("MAIN: exit");
}

///////////////////////////////////////////////////////////////////////////////////////////////

fn test_mutex(){
    // Создаем переменну, которая синхронизована мьютексом
    // Mutex - по сути это shared_ptr
    let m = Mutex::new(5);
    {
        // Получаем изменяемую переменную-указатель на данные
        let mut num = m.lock().unwrap();
        // Изменяем значение
        *num = 6;
        // Разблокировка будет автоматическая, так как мы вышли из области видимости
    }
    println!("m = {:?}", m);

    // Создаем переменную, защищенную мьютексом
    // причем - для работы сразу с несколькими переменными требуется
    // счетчик ссылок
    let counter = Arc::new(Mutex::new(0));
    // Вектор с обработчиками
    let mut handles = vec![];

    for _ in 0..10 {
        // Создаем копию счетчика ссылок для каждого потока
        let counter = counter.clone();
        let handle = thread::spawn(move || {
            // Получаем изменяемую переменную-указатель на данные
            let mut num = counter.lock().unwrap();
            // Изменяем значение
            *num += 1;
            // Разблокировка будет автоматическая, так как мы вышли из области видимости
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    println!("Result: {}", *counter.lock().unwrap());
}

///////////////////////////////////////////////////////////////////////////////////////////////

fn main() {
    //test_variables_move();
    //test_channel_1();
    test_channel_2();
}