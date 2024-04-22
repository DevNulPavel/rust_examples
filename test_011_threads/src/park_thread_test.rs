use std::thread;
use std::time::Duration;
use std::sync::{
    atomic::{
        AtomicBool, 
        Ordering
    },
    Arc,
};

pub fn park_thread_test_1() {
    let parked_thread = thread::Builder::new()
        .spawn(|| {
            println!("THREAD: Parking thread");
            // Данный поток паркуется и ждет, когда станет возможным снова продолжить работу после распарковки
            // Данный метод можно вызывать только после того, как был вызван метод unpark
            // Однако - есть проблема, токен может восстанавливаться случайным образом, так называемые spurious wakeupds
            //     поэтому желательно, делать дополнительные проверки с помощью Atomic переменных
            thread::park();
            println!("THREAD: Thread unparked");
        })
        .unwrap();

    // Let some time pass for the thread to be spawned.
    thread::sleep(Duration::from_millis(10));

    println!("MAIN: Unpark the thread");
    // Данный вызов делает токет доступным, если его еще не было
    parked_thread.thread().unpark();

    parked_thread.join().unwrap();
}

pub fn park_thread_test_2() {
    let flag_src = Arc::new(AtomicBool::new(false));
    let flag_for_thread = Arc::clone(&flag_src);

    let parked_thread = thread::spawn(move || {
        // Дожидаемся, пока флаг нельзя будет получить, это нужно, чтобы избежать spurious wakeups
        while !flag_for_thread.load(Ordering::Acquire) {
            println!("Parking thread");
            // Паркуемся и ждем
            thread::park();
            println!("Thread unparked");
        }
        println!("Flag received");
    });
    
    thread::sleep(Duration::from_millis(10));
    
    // Выставляем флаг
    flag_src.store(true, Ordering::Release);
    
    println!("Unpark the thread");
    // Вызываем распарковку для восстановления работы потока
    parked_thread.thread().unpark();

    parked_thread.join().unwrap();
}
