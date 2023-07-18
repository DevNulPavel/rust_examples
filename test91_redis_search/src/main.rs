use std::{str::from_utf8, time::SystemTime};

use rand::Rng;
use redis::{aio::ConnectionManager, cmd, pipe, Client};

#[tokio::main]
async fn main() {
    // Путь к sock файлику редиса
    let sock_file_path = std::env::current_dir().unwrap().join("redis/redis.sock");

    // Создаем клиента - он по своей сути является лишь информацией о соединении
    let client = Client::open(format!("redis+unix://{}", sock_file_path.display())).unwrap();

    // Менеджер коннектов
    let mut connection_manager = client.get_tokio_connection_manager().await.unwrap();

    // Пробуем пропинговать сервер редиса, для того, чтобы проверить, что он готов и все загрузил
    ping_redis(&mut connection_manager).await;

    // Удаляем индекс старый если есть
    cmd("FT.DROPINDEX")
        .arg("test_index")
        .arg("DD")
        .query_async::<_, ()>(&mut connection_manager)
        .await
        .ok();

    // Создаем снова индекс
    cmd("FT.CREATE")
        .arg("test_index")
        .arg("ON")
        .arg("HASH")
        .arg("PREFIX")
        .arg("1")
        .arg("t:")
        .arg("SCHEMA")
        .arg("l")
        .arg("NUMERIC")
        .arg("SORTABLE")
        // .arg("i")
        // .arg("NUMERIC")
        // .arg("SORTABLE")
        .query_async::<_, ()>(&mut connection_manager)
        .await
        .unwrap();

    println!("Index created");

    let mut rand_generator = rand::thread_rng();

    {
        let now_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        const TOTAL_COUNT: usize = 1_000_000;
        const BUNCH_SIZE: usize = 1_000;

        for i in 0..(TOTAL_COUNT / BUNCH_SIZE) {
            let mut pipe = pipe();

            for j in 0..BUNCH_SIZE {
                let key = format!("t:{i}:{j}");
                let last_check = now_timestamp - rand_generator.gen_range(600..60000);
                let important: u8 = rand_generator.gen_range(0..=1);

                let mut command = cmd("HSET");

                command
                    .arg(key.as_str())
                    .arg("l")
                    .arg(last_check)
                    .arg("i")
                    .arg(important);

                pipe.add_command(command);
            }

            pipe.query_async::<_, ()>(&mut connection_manager)
                .await
                .unwrap();
        }
    }

    println!("Test data written");

    {
        let now_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let begin = SystemTime::now();

        let mut found_count = 0;
        let mut change_count = 0;

        const SEARCH_COUNT: usize = 1_000;

        for _ in 0..SEARCH_COUNT {
            // let last_check_start = now_timestamp - 60000;
            let last_check = now_timestamp - rand_generator.gen_range(600..60000);
            let important = rand_generator.gen_range(0..=1);

            // let query = format!("'@l:[-inf {0}] @i:[{1} {1}]'", last_check, important);
            let query = format!("'@l:[-inf {0}]'", last_check);
            // let query = "*";

            let search_resp = cmd("FT.SEARCH")
                .arg("test_index")
                .arg(query)
                .arg("NOCONTENT")
                // .arg("FILTER")
                // .arg("l")
                // .arg("-inf")
                // .arg(last_check)
                .arg("SORTBY")
                .arg("l")
                .arg("ASC")
                .arg("LIMIT")
                .arg("0")
                .arg("1")
                .query_async::<_, redis::Value>(&mut connection_manager)
                .await
                .unwrap();

            let found_key = deserialize_resp(&search_resp);

            if let Some(_found_key) = found_key {
                found_count += 1;

                // cmd("HSET")
                //     .arg(found_key)
                //     .arg("l")
                //     .arg(now_timestamp)
                //     .query_async::<_, ()>(&mut connection_manager)
                //     .await
                //     .unwrap();
                // change_count += 1;
            }
        }

        let duration = SystemTime::now().duration_since(begin).unwrap().as_millis();

        println!(
            "AVG search and update: {} mSec.",
            cast::f64(duration) / cast::f64(SEARCH_COUNT)
        );

        println!("Found count: {found_count}");
        assert!(found_count > 0);

        println!("Updates count: {change_count}");
        // assert!(change_count > 0);
    }
}

async fn ping_redis(connection_manager: &mut ConnectionManager) {
    for i in 0.. {
        let res = cmd("ping")
            .query_async::<_, redis::Value>(connection_manager)
            .await;

        match res {
            Ok(res) => match res {
                redis::Value::Status(res) if res.to_lowercase() == "pong" => {
                    break;
                }
                res => {
                    panic!("Invalid result: {:?}", res);
                }
            },
            Err(err) => {
                if i < 30 {
                    eprintln!("Redis is not ready, awaiting: {}", err);
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                } else {
                    panic!("Redis is not ready: {}", err)
                }
            }
        }
    }
}

// Десериализация ответа на запрос следующего элемента очереди
pub fn deserialize_resp(value: &redis::Value) -> Option<&str> {
    let mut values_iter = value.as_sequence().unwrap().iter();

    // Первый элемент - количество элементов в хэше
    let count = match values_iter.next().unwrap() {
        redis::Value::Int(c) => *c,
        _ => panic!(),
    };

    // Нет элементов
    if count == 0 {
        return None;
    }

    // Следующий элемент - это ключ
    let key = match values_iter.next() {
        Some(redis::Value::Data(data)) => from_utf8(data).unwrap(),
        _ => panic!(),
    };

    Some(key)
}
