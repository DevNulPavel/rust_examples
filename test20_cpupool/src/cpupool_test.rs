extern crate futures;
extern crate futures_cpupool;

use futures::future::Future;

macro_rules! print_ptread_name {
    () => {
        let handle = std::thread::current();
        println!("Line {}: {:?}", std::line!(), handle.name().unwrap_or("unknown"));
    };
}

// Фактически - аналог подхода с тасками

pub fn test_tasks(){
    print_ptread_name!();

    // Создаем пул потоков, на котором у нас непосредственно будут выполняться наши задачи
    let pool_1 = futures_cpupool::Builder::new()
        .pool_size(1)
        .stack_size(1024*32)
        .name_prefix("Actor 1: ")
        .create();

    let pool_2 = futures_cpupool::Builder::new()
        .pool_size(1)
        .stack_size(1024*32)
        .name_prefix("Actor 2: ")
        .create();

    let pool_3 = futures_cpupool::Builder::new()
        .pool_size(1)
        .stack_size(1024*32)
        .name_prefix("Actor 3: ")
        .create();

    // Создаем пул потоков, на котором у нас непосредственно будут выполняться наши задачи
    //let pool_2 = futures_cpupool::CpuPool::new(1);

    // spawn() метод значит, что переданная как параметр фьюча будет прикрепляться и выполняться на конкретном указанном пуле потоков
    // pool_1.spawn(b) - заново перекидывать задачу нельзя после вызова spawn, тогда задача потеряется

    // Запускаем некоторую работу в потоке, при этом создавая новую фьючу
    // Нужно вручную указывать тип переменных, иначе компилятор ничего не понимает
    let a = pool_1.spawn_fn(move ||->Result<i32, ()>{
        print_ptread_name!();
        //std::thread::sleep(std::time::Duration::from_millis(1000));
        Ok(100) // Воздвращает Result
    });

    // Затем мы продолжаем исполнение задачи в потоке 2
    let b = pool_2.spawn(a.and_then(move |prev_result| {
        print_ptread_name!();
        // std::thread::sleep(std::time::Duration::from_millis(1000));
        Ok(prev_result + 100)
    }));

    // Но можно дополнтительно обработать ошибку
    let c = pool_1.spawn(b.then(|res|->Result<i32, i32>{
        print_ptread_name!();
        if let Ok(prev_res) = res{
            // std::thread::sleep(std::time::Duration::from_millis(1000));
            Ok(prev_res + 10)
        }else{
            Ok(0)
        }
    }));

    // Создаем другую задачу
    let d = pool_3.spawn_fn(||-> Result<i32, i32> {
        print_ptread_name!();
        std::thread::sleep(std::time::Duration::from_millis(2000));
        Err(123)
    })
    // Перегоняем ошибку в новый тип
    .map_err(|err|{
        err + 1
    })
    // Если предыдущая случилась с ошибкой, запускает новую задачу
    .or_else(|err|-> Result<i32, i32>{
        std::thread::sleep(std::time::Duration::from_millis(1000));
        Ok(err)
    })
    // Перегоняем в новый тип успешный результат
    .map(|val|->i32{
        val + 10
    });

    // Соединяем результаты C и D
    let joined = c.join(d);

    // Вычисляем результат
    let sum = joined.map(|(a, b)| {
        a + b
    });

    // Получаем сумму результатов c блокировкой
    let result = sum.wait().unwrap();

    // Print out the result
    println!("{:?}", result);

    // let b = a.and_then();

    //let future = async { Ok::<i32, i32>(1) };
    //let future = future.and_then(|x| async move { Ok::<i32, i32>(x + 3) });
    // assert_eq!(future.await, Ok(4));
    
    // Можно создать сразу зарезолвленную фьючу
    // let future_of_1 = future::ok::<u32, u32>(1);
    // let future_of_4 = future_of_1.and_then(|x| {
    //     Ok(x + 3)
    // });
    
    // let future_of_err_1 = future::err::<u32, u32>(1);

    // future_of_err_1.and_then(|_| -> FutureResult<u32, u32> {
    //     panic!("should not be called in case of an error");
    // });
}