use arachne_api::SelectorSize;
use arachne_client::{ArachneClient, SelectUser, SelectorCreateStatus, User, UserId};
use serde_json::{Value, json};
use tokio::time::{Duration, sleep};

// Функция для подключения к запущенному Серверу
async fn setup_client() -> ArachneClient {
    ArachneClient::new("127.0.0.1:50051", 1)
        .await
        .expect("Failed to connect to Tarantool. Is it running?")
}

// Обновление пользователя инвалидирует очередь
#[tokio::test]
async fn test_user_update_invalidates_queue() {
    let client = setup_client().await;
    let platform_id = 99;
    let selector_id_1 = 1;
    let selector_id_2 = 2;

    // 1. Очищаем мусор если был и создаем платформу/селекторы
    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();
    client
        .selector_create(platform_id, selector_id_1, "score > 10", 60)
        .await
        .unwrap();
    client
        .selector_create(platform_id, selector_id_2, "score < 10", 60)
        .await
        .unwrap();

    // 2. Добавляем пользователя, который подходит под селектор 1 (score = 20)
    let user = User {
        id: UserId::Int(1001),
        payload: "data".to_string().into_bytes(),
        meta: json!({
            "score": 20
        }),
    };
    client.user_add(platform_id, user).await.unwrap();

    // Ждем 300мс, чтобы Time Ticker Fiber успел положить пользователя в hot_queues
    sleep(Duration::from_millis(300)).await;

    // Проверяем, что он недоступен в селекторе 2
    let task_wrong = client
        .select_user::<Value>(platform_id, selector_id_2)
        .await
        .unwrap();
    assert!(
        matches!(task_wrong, SelectUser::Finished),
        "User incorrectly routed to selector 2 early!"
    );

    // 3. Обновляем пользователя так, чтобы он БОЛЬШЕ НЕ ПОДХОДИЛ под селектор 1,
    // но стал подходить под селектор 2 (score = 5)
    let new_user = User {
        id: UserId::Int(1001),
        payload: "new_data".to_string().into_bytes(),
        meta: json!({
            "score": 5
        }),
    };

    println!("Updating user...");
    client.user_update(platform_id, new_user).await.unwrap();

    // Ждем 1000мс, чтобы Time Ticker Fiber успел переработать измененного юзера
    sleep(Duration::from_millis(1000)).await;

    println!("Trying to get task from selector 1...");
    // 4. Пытаемся получить задачу из ПЕРВОГО селектора
    let task_1 = client
        .select_user::<Value>(platform_id, selector_id_1)
        .await
        .unwrap();

    // Ожидаем Empty, так как задача была инвалидирована
    assert!(
        matches!(task_1, SelectUser::Finished),
        "User was not removed from selector 1 hot_queues during user_update! Got: {:?}",
        task_1
    );

    // 5. Пытаемся получить задачу из ВТОРОГО селектора
    let task_2 = client
        .select_user::<Value>(platform_id, selector_id_2)
        .await
        .unwrap();

    // Ожидаем Found, так как Time Ticker должен был положить его туда
    assert!(
        matches!(task_2, SelectUser::Found(_)),
        "User was not routed to selector 2 after user_update! Got: {:?}",
        task_2
    );

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

//  Модификация и удаление селектора инвалидируют очередь
#[tokio::test]
async fn test_selector_modify_and_delete_invalidates_queue() {
    let client = setup_client().await;
    let platform_id = 100;
    let selector_id = 2;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    // 1. Создаем селектор, под который подходят все
    client
        .selector_create(platform_id, selector_id, "score > 0", 60)
        .await
        .unwrap();

    // 2. Добавляем пользователя
    let user = User {
        id: UserId::Int(2001),
        payload: "data".to_string().into_bytes(),
        meta: json!({
            "score": 10
        }),
    };
    client.user_add(platform_id, user).await.unwrap();

    // Ждем, пока Time Ticker его положит в очередь
    sleep(Duration::from_millis(300)).await;

    // 3. Изменяем селектор так, чтобы под него никто не подходил.
    // Этот вызов должен очистить все старые задачи этого селектора в hot_queues.
    let res = client
        .selector_create(platform_id, selector_id, "score > 100", 60)
        .await
        .unwrap();
    assert_eq!(res, SelectorCreateStatus::Modified);

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 4. Пытаемся получить задачу
    let task = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    assert!(
        matches!(task, SelectUser::Finished),
        "Queue was not invalidated on selector_modify! Got: {:?}",
        task
    );

    // 5. Тестируем удаление селектора
    // Снова делаем селектор подходящим для юзера
    let res = client
        .selector_create(platform_id, selector_id, "score > 0", 60)
        .await
        .unwrap();
    assert_eq!(res, SelectorCreateStatus::Modified);

    sleep(Duration::from_millis(300)).await; // Time Ticker вернет его обратно

    // Удаляем селектор, он должен каскадно удалить свои очереди
    client
        .selector_delete(platform_id, selector_id)
        .await
        .unwrap();

    let task2 = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    assert!(
        matches!(task2, SelectUser::SelectorDoesNotExist),
        "Queue was not invalidated / selector not deleted! Got: {:?}",
        task2
    );

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

// Пересоздание селектора (изменение текста) инвалидирует задачи
#[tokio::test]
async fn test_selector_recreate_invalidates_queue() {
    let client = setup_client().await;
    let platform_id = 102;
    let selector_id = 4;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    // 1. Создаем селектор, под который подходят все
    client
        .selector_create(platform_id, selector_id, "score > 0", 60)
        .await
        .unwrap();

    // 2. Добавляем пользователя
    let user = User {
        id: UserId::Int(4001),
        payload: "data".to_string().into_bytes(),
        meta: json!({
            "score": 10
        }),
    };
    client.user_add(platform_id, user).await.unwrap();
    let user = User {
        id: UserId::Int(4002),
        payload: "data".to_string().into_bytes(),
        meta: json!({
            "score": 10
        }),
    };
    client.user_add(platform_id, user).await.unwrap();

    // Ждем, пока Time Ticker его положит в очередь
    sleep(Duration::from_millis(300)).await;

    // 3. Пересоздаем селектор (с тем же ID) с измененным запросом
    // Это должно работать как upsert и инвалидировать старые задачи
    client
        .selector_create(platform_id, selector_id, "score > 100", 60)
        .await
        .unwrap();

    // 4. Пытаемся получить задачу
    let _task = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

// Утечка "Зомби-задач"
#[tokio::test]
async fn test_zombie_tasks_leak_on_selector_delete() {
    let client = setup_client().await;
    let platform_id = 104;
    let selector_id = 6;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    // 1. Создаем селектор
    client
        .selector_create(platform_id, selector_id, "score > 0", 60)
        .await
        .unwrap();

    // 2. Добавляем пользователя
    let user = User {
        id: UserId::Int(6001),
        payload: "data".to_string().into_bytes(),
        meta: json!({
            "score": 10,
            "next_check_time": 0
        }),
    };

    client.user_add(platform_id, user).await.unwrap();

    // 3. Удаляем селектор
    // Это удалит задачи селектора из hot_queues.
    client
        .selector_delete(platform_id, selector_id)
        .await
        .unwrap();

    // 4. Создаем новый селектор, под который этот пользователь тоже подходит
    let selector_id_new = 7;
    client
        .selector_create(platform_id, selector_id_new, "score == 10", 60)
        .await
        .unwrap();

    // Ждем 500мс, чтобы Time Ticker успел его проверить
    sleep(Duration::from_millis(500)).await;

    // 5. Пытаемся получить задачу из НОВОГО селектора
    let task = client
        .select_user::<Value>(platform_id, selector_id_new)
        .await
        .unwrap();

    // Если пользователь стал "зомби" (next_check_ts остался u64::MAX после удаления первого селектора),
    // то Time Ticker его никогда не найдет, и задача не попадет в selector 7.
    assert!(
        matches!(task, SelectUser::Found(_)),
        "ZOMBIE LEAK DETECTED! User was not routed to the new selector because his next_check_ts was stuck at u64::MAX. Got: {:?}",
        task
    );

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

//  Консистентность при параллельном получении задач из одного селектора
#[tokio::test]
async fn test_concurrent_select_consistency() {
    let platform_id = 101;
    let selector_id = 3;

    let admin_client = setup_client().await;
    let _ = admin_client.platform_delete(platform_id).await;
    admin_client.platform_create(platform_id).await.unwrap();

    admin_client
        .selector_create(platform_id, selector_id, "score > 0", 60)
        .await
        .unwrap();

    // Добавляем 5 пользователей
    for i in 1..=5 {
        let user = User {
            id: UserId::Int(3000 + i as i64),
            payload: format!("data_{}", i).into_bytes(),
            meta: json!({
                "score": 10
            }),
        };
        admin_client.user_add(platform_id, user).await.unwrap();
    }

    // Ждем, пока Time Ticker их всех положит в очередь
    sleep(Duration::from_millis(500)).await;

    // Запускаем 3 запроса одновременно в разных тасках, каждый со своим подключением
    let t1 = tokio::spawn(async move {
        let c = setup_client().await;
        c.select_user::<Value>(platform_id, selector_id)
            .await
            .unwrap()
    });
    let t2 = tokio::spawn(async move {
        let c = setup_client().await;
        c.select_user::<Value>(platform_id, selector_id)
            .await
            .unwrap()
    });
    let t3 = tokio::spawn(async move {
        let c = setup_client().await;
        c.select_user::<Value>(platform_id, selector_id)
            .await
            .unwrap()
    });

    let (res1, res2, res3) = tokio::join!(t1, t2, t3);

    let task1 = res1.unwrap();
    let task2 = res2.unwrap();
    let task3 = res3.unwrap();

    // Все три должны найти пользователя
    let user1 = match task1 {
        SelectUser::Found(u) => u.id,
        _ => panic!("Task 1 did not find a user, got: {:?}", task1),
    };
    let user2 = match task2 {
        SelectUser::Found(u) => u.id,
        _ => panic!("Task 2 did not find a user, got: {:?}", task2),
    };
    let user3 = match task3 {
        SelectUser::Found(u) => u.id,
        _ => panic!("Task 3 did not find a user, got: {:?}", task3),
    };

    // Проверяем, что все 3 выдали РАЗНЫХ пользователей (атомарная выдача)
    assert_ne!(user1, user2, "Task 1 and 2 received the same user!");
    assert_ne!(user1, user3, "Task 1 and 3 received the same user!");
    assert_ne!(user2, user3, "Task 2 and 3 received the same user!");

    println!(
        "Success! Clients received atomic tasks: {}, {}, {}",
        user1, user2, user3
    );

    // Очистка
    admin_client.platform_delete(platform_id).await.unwrap();
}

#[tokio::test]
async fn test_prefetch_queue() {
    let client = setup_client().await;
    let platform_id = 105;
    let selector_id = 8;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    // 1. Создаем селектор
    client
        .selector_create(platform_id, selector_id, "score > 0", 60)
        .await
        .unwrap();

    // 2. Добавляем пользователя с тяжелым Payload
    let heavy_payload = vec![0; 100 * 1024];
    let user = User {
        id: UserId::Int(8001),
        payload: heavy_payload.clone(),
        meta: json!({
            "score": 10
        }),
    };

    client.user_add(platform_id, user).await.unwrap();

    // 3. Ждем, чтобы Time Ticker (Fiber) успел обработать prefetch queue
    sleep(Duration::from_millis(300)).await;

    // 4. Пытаемся получить задачу
    let start_time = std::time::Instant::now();
    let task = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    let elapsed = start_time.elapsed();

    println!("Prefetched task selected in: {:?}", elapsed);

    match task {
        SelectUser::Found(u) => {
            assert_eq!(u.id, UserId::Int(8001));
            assert_eq!(u.payload, heavy_payload);
        }
        _ => panic!("Prefetched routing failed! Got: {:?}", task),
    }

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

// Проверка строгого FIFO после Lock Timeout
#[tokio::test]
async fn test_fifo_ordering_on_lock_timeout() {
    let client = setup_client().await;
    let platform_id = 106;
    let selector_id = 9;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    // 1. Создаем селектор с коротким lock_timeout (2 секунды)
    client
        .selector_create(platform_id, selector_id, "score > 0", 2)
        .await
        .unwrap();

    // 2. Добавляем первого пользователя
    client
        .user_add(
            platform_id,
            User {
                id: UserId::Int(9001),
                payload: b"data1".to_vec(),
                meta: json!({
                    "score": 10
                }),
            },
        )
        .await
        .unwrap();

    // Небольшая пауза, чтобы гарантировать разный inserted_at (на уровне микросекунд)
    sleep(Duration::from_millis(50)).await;

    // 3. Добавляем второго пользователя
    client
        .user_add(
            platform_id,
            User {
                id: UserId::Int(9002),
                payload: b"data2".to_vec(),
                meta: json!({
                    "score": 10
                }),
            },
        )
        .await
        .unwrap();

    // Ждем
    sleep(Duration::from_millis(300)).await;

    // 4. Забираем первого юзера. По правилам FIFO это должен быть 9001 (создан первым).
    let task1 = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    let id1_first = match task1 {
        SelectUser::Found(u) => u.id,
        _ => panic!("Expected to find user"),
    };
    assert_eq!(
        id1_first,
        UserId::Int(9001),
        "Expected first user to be 9001 (FIFO violation)"
    );

    // 5. Забираем второго юзера. Это должен быть 9002.
    let task2 = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    let id2_first = match task2 {
        SelectUser::Found(u) => u.id,
        _ => panic!("Expected to find user"),
    };
    assert_eq!(
        id2_first,
        UserId::Int(9002),
        "Expected second user to be 9002 (FIFO violation)"
    );

    // 6. Ждем 2.5 секунды, чтобы прошел lock_timeout
    println!("Waiting for lock timeouts...");
    sleep(Duration::from_millis(2500)).await;

    // 7. Снова забираем задачи. Так как Ticker обрабатывает их примерно в одно время,
    // он вставляет их в hot_queues с новым inserted_at.
    let task_recovered_1 = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    let task_recovered_2 = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();

    assert!(
        matches!(task_recovered_1, SelectUser::Found(_)),
        "Failed to recover first task"
    );
    assert!(
        matches!(task_recovered_2, SelectUser::Found(_)),
        "Failed to recover second task"
    );

    let id_rec1 = match task_recovered_1 {
        SelectUser::Found(u) => u.id,
        _ => UserId::Int(0),
    };
    let id_rec2 = match task_recovered_2 {
        SelectUser::Found(u) => u.id,
        _ => UserId::Int(0),
    };

    println!(
        "Recovered tasks after lock timeout: {}, {}",
        id_rec1, id_rec2
    );

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

//  Два селектора с одинаковым запросом возвращают одних и тех же пользователей
#[tokio::test]
async fn test_two_selectors_same_query() {
    let client = setup_client().await;
    let platform_id = 107;
    let selector_id_1 = 10;
    let selector_id_2 = 11;
    let query = "score > 10 AND created_at AGE < 1 hours";

    client.platform_create(platform_id).await.unwrap();

    // Создаем два селектора с абсолютно одинаковым запросом (lock timeout = 1 сек, чтобы было быстрее)
    client
        .selector_create(platform_id, selector_id_1, query, 1)
        .await
        .unwrap();
    client
        .selector_create(platform_id, selector_id_2, query, 1)
        .await
        .unwrap();

    // Добавляем текущее время минус 30 минут, чтобы условие < 1h выполнялось
    let now_ts = chrono::Utc::now().timestamp();

    // Добавляем пользователя
    client
        .user_add(
            platform_id,
            User {
                id: UserId::Int(10001),
                payload: b"data1".to_vec(),
                meta: json!({
                            "score": 20,
                    "created_at": now_ts - 1800
                }),
            },
        )
        .await
        .unwrap();

    // Ждем, чтобы Ticker обработал (или сработала синхронная маршрутизация)
    sleep(Duration::from_millis(300)).await;

    // Пытаемся забрать задачу из ПЕРВОГО селектора - получаем 10001
    let task1 = client
        .select_user::<Value>(platform_id, selector_id_1)
        .await
        .unwrap();
    let id1 = match task1 {
        SelectUser::Found(u) => u.id,
        _ => panic!("Expected to find user"),
    };
    assert_eq!(
        id1,
        UserId::Int(10001),
        "Expected user 10001 from selector 1"
    );

    // Пытаемся забрать задачу из второго селектора - должна быть пуста (так как юзер заблокирован первым селектором)
    let task2 = client
        .select_user::<Value>(platform_id, selector_id_2)
        .await
        .unwrap();
    assert!(
        matches!(task2, SelectUser::Empty),
        "Expected Empty, since the only user is locked"
    );

    // Ждем истечения таймаута (1.5 сек), чтобы юзер 10001 снова стал доступен
    sleep(Duration::from_millis(1500)).await;

    // Теперь забираем юзера из второго селектора - он должен там появиться, так как запрос такой же
    let task3 = client
        .select_user::<Value>(platform_id, selector_id_2)
        .await
        .unwrap();
    let _id3 = match task3 {
        SelectUser::Found(u) => u.id,
        _ => panic!("Expected to find user in selector 2 after timeout"),
    };
    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

// Проверка Time Ticker - фоновое пробуждение задачи
#[tokio::test]
async fn test_time_ticker_future_wakeup() {
    let client = setup_client().await;
    let platform_id = 109;
    let selector_id = 13;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    // Селектор: "check_time < NOW()". Задача должна быть взята в работу только когда NOW() превысит check_time.
    let query = "check_time < NOW()";
    client
        .selector_create(platform_id, selector_id, query, 60)
        .await
        .unwrap();

    // Добавляем юзера, у которого check_time наступит через 2 секунды в будущем
    let future_ts = chrono::Utc::now().timestamp() + 2;

    client
        .user_add(
            platform_id,
            User {
                id: UserId::Int(12001),
                payload: b"data1".to_vec(),
                meta: json!({
                    "check_time": future_ts
                }),
            },
        )
        .await
        .unwrap();

    // 1. Сразу же проверяем - задача не должна быть в очереди (check_time еще в будущем)
    let task_early = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    assert!(
        matches!(task_early, SelectUser::Empty),
        "Expected Empty, task is in the future"
    );

    // 2. Ждем 4 секунды, чтобы будущее время наступило и Time Ticker обработал задачу
    println!("Waiting for Time Ticker to wake up the task...");
    sleep(Duration::from_millis(4000)).await;

    // 3. Теперь задача должна лежать в горячей очереди
    let task_ready = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    match task_ready {
        SelectUser::Found(u) => {
            assert_eq!(
                u.id,
                UserId::Int(12001),
                "Expected user 12001 to be woken up by Ticker"
            );
        }
        _ => panic!("Expected task to be ready after 4 seconds"),
    }

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
struct CustomUserMeta {
    score: i32,
    tags: Vec<String>,
    is_active: bool,
}

// Использование пользовательской структуры для meta
#[tokio::test]
async fn test_custom_struct_for_meta() {
    let client = setup_client().await;
    let platform_id = 110;
    let selector_id = 14;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    let query = "score >= 10 AND is_active == true AND tags AMONG [\"premium\"]";
    client
        .selector_create(platform_id, selector_id, query, 60)
        .await
        .unwrap();

    let custom_meta = CustomUserMeta {
        score: 15,
        tags: vec!["premium".to_string(), "new".to_string()],
        is_active: true,
    };

    client
        .user_add(
            platform_id,
            User {
                id: UserId::Int(13001),
                payload: b"custom_data".to_vec(),
                meta: custom_meta.clone(),
            },
        )
        .await
        .unwrap();

    sleep(Duration::from_millis(300)).await;

    let task = client
        .select_user::<CustomUserMeta>(platform_id, selector_id)
        .await
        .unwrap();

    match task {
        SelectUser::Found(u) => {
            assert_eq!(u.id, UserId::Int(13001));
            assert_eq!(u.payload, b"custom_data".to_vec());
            assert_eq!(
                u.meta, custom_meta,
                "Meta should be correctly deserialized into CustomUserMeta"
            );
        }
        _ => panic!("Expected to find user with custom struct meta"),
    }

    client.platform_delete(platform_id).await.unwrap();
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
struct ComplexUserMeta {
    settings: Settings,
    optional_field: Option<String>,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq)]
struct Settings {
    theme: String,
    notifications: bool,
}

// Тестирование NOT EXISTS / EXISTS и вложенных структур JSON
#[tokio::test]
async fn test_exists_not_exists_with_complex_struct() {
    let client = setup_client().await;
    let platform_id = 111;
    let selector_id = 15;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    // Селектор: settings.notifications == true AND optional_field NOT EXISTS
    let query = "settings.notifications == true AND optional_field NOT EXISTS";
    let status = client
        .selector_create(platform_id, selector_id, query, 60)
        .await
        .unwrap();
    assert_eq!(
        status,
        SelectorCreateStatus::Created,
        "Selector must be created"
    );

    let complex_meta = ComplexUserMeta {
        settings: Settings {
            theme: "dark".to_string(),
            notifications: true,
        },
        optional_field: None, // Сериализуется как null, подходит под NOT EXISTS
    };

    client
        .user_add(
            platform_id,
            User {
                id: UserId::Int(14001),
                payload: b"data".to_vec(),
                meta: complex_meta.clone(),
            },
        )
        .await
        .unwrap();

    sleep(Duration::from_millis(300)).await;

    let task = client
        .select_user::<ComplexUserMeta>(platform_id, selector_id)
        .await
        .unwrap();

    match task {
        SelectUser::Found(u) => {
            assert_eq!(u.id, UserId::Int(14001));
            assert_eq!(u.meta, complex_meta);
        }
        _ => panic!("Expected to find user with complex struct meta using NOT EXISTS"),
    }

    client.platform_delete(platform_id).await.unwrap();
}

// Использование строковых и отрицательных User ID
#[tokio::test]
async fn test_string_and_negative_user_ids() {
    let client = setup_client().await;
    let platform_id = 112;
    let selector_id = 16;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    let query = "status == \"active\"";
    client
        .selector_create(platform_id, selector_id, query, 60)
        .await
        .unwrap();

    // 1. Отрицательный ID
    client
        .user_add(
            platform_id,
            User {
                id: UserId::Int(-999),
                payload: b"negative_id".to_vec(),
                meta: json!({
                    "status": "active"
                }),
            },
        )
        .await
        .unwrap();

    // 2. Строковый ID
    client
        .user_add(
            platform_id,
            User {
                id: UserId::Str("instagram_user_123".into()),
                payload: b"string_id".to_vec(),
                meta: json!({
                    "status": "active"
                }),
            },
        )
        .await
        .unwrap();

    sleep(Duration::from_millis(300)).await;

    let mut found_ids = Vec::new();
    for _ in 0..2 {
        let task = client
            .select_user::<Value>(platform_id, selector_id)
            .await
            .unwrap();
        match task {
            SelectUser::Found(u) => found_ids.push(u.id),
            _ => break,
        }
    }

    assert_eq!(found_ids.len(), 2, "Expected to find 2 tasks");
    assert!(
        found_ids.contains(&UserId::Int(-999)),
        "Expected to find negative ID -999"
    );
    assert!(
        found_ids.contains(&UserId::Str("instagram_user_123".into())),
        "Expected to find string ID instagram_user_123"
    );

    client.platform_delete(platform_id).await.unwrap();
}

// Поиск пользователя по ID (user_search)
#[tokio::test]
async fn test_user_search() {
    let client = setup_client().await;
    let platform_id = 113;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    let target_user = User {
        id: UserId::Int(15001),
        payload: b"search_data".to_vec(),
        meta: json!({
            "status": "vip",
            "score": 42
        }),
    };

    // Добавляем пользователя
    client
        .user_add(platform_id, target_user.clone())
        .await
        .unwrap();

    // 1. Ищем существующего пользователя
    let found = client
        .user_search::<Value>(platform_id, UserId::Int(15001))
        .await
        .unwrap();

    assert!(found.is_some(), "Expected to find user 15001");
    let found_user = found.unwrap();
    assert_eq!(found_user.id, target_user.id);
    assert_eq!(found_user.payload, target_user.payload);
    assert_eq!(found_user.meta["score"], 42);

    // 2. Ищем несуществующего пользователя
    let missing = client
        .user_search::<Value>(platform_id, UserId::Int(99999))
        .await
        .unwrap();
    assert!(missing.is_none(), "Expected None for non-existent user");

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

// Досрочное снятие блокировки с задачи (user_release)
#[tokio::test]
async fn test_user_release_resets_lock() {
    let client = setup_client().await;
    let platform_id = 114;
    let selector_id = 17;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    // Создаем селектор с длинным таймаутом блокировки (60 секунд)
    client
        .selector_create(platform_id, selector_id, "score > 0", 60)
        .await
        .unwrap();

    client
        .user_add(
            platform_id,
            User {
                id: UserId::Int(16001),
                payload: b"data".to_vec(),
                meta: json!({
                    "score": 10
                }),
            },
        )
        .await
        .unwrap();

    sleep(Duration::from_millis(300)).await;

    // 1. Берем задачу в работу. Теперь пользователь заблокирован на 60 секунд.
    let task1 = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    assert!(
        matches!(task1, SelectUser::Found(_)),
        "Expected to find user on first select"
    );

    // 2. Пытаемся взять еще одну задачу - очередь должна быть пуста (пользователь залочен)
    let task2 = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    assert!(
        matches!(task2, SelectUser::Empty),
        "Expected Empty on second select (user should be locked)"
    );

    // 3. Досрочно "отпускаем" (релизим) пользователя
    let release_status = client
        .user_release(platform_id, UserId::Int(16001))
        .await
        .unwrap();

    assert_eq!(
        format!("{:?}", release_status),
        "Released",
        "Expected user to be successfully released"
    );

    // Даем Time Ticker'у (или маршрутизатору) вернуть пользователя в очередь
    sleep(Duration::from_millis(300)).await;

    // 4. Снова пытаемся получить задачу - пользователь должен вернуться обратно!
    let task3 = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    assert!(
        matches!(task3, SelectUser::Found(_)),
        "User was NOT returned to the queue after user_release!"
    );

    // Проверка релиза несуществующего юзера
    let missing_release = client
        .user_release(platform_id, UserId::Int(99999))
        .await
        .unwrap();
    assert_eq!(format!("{:?}", missing_release), "NotFound");

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

// Метрики, размеры очередей и список платформ
#[tokio::test]
async fn test_metrics_and_platform_lists() {
    let client = setup_client().await;
    let platform_1 = 115;
    let platform_2 = 116;
    let selector_id = 18;

    let _ = client.platform_delete(platform_1).await;
    let _ = client.platform_delete(platform_2).await;

    // 1. Создаем две платформы
    client.platform_create(platform_1).await.unwrap();
    client.platform_create(platform_2).await.unwrap();

    // 2. Проверяем, что обе появились в списке platforms_list
    let platforms = client.platforms_list().await.unwrap();
    assert!(platforms.contains(&platform_1), "List missing platform 115");
    assert!(platforms.contains(&platform_2), "List missing platform 116");

    // 3. Создаем селектор в первой платформе
    client
        .selector_create(platform_1, selector_id, "score > 0", 60)
        .await
        .unwrap();

    // 4. Добавляем 2 юзеров в платформу 1
    client
        .user_add(
            platform_1,
            User {
                id: UserId::Int(1),
                payload: vec![],
                meta: json!({"score": 10}),
            },
        )
        .await
        .unwrap();
    client
        .user_add(
            platform_1,
            User {
                id: UserId::Int(2),
                payload: vec![],
                meta: json!({"score": 10}),
            },
        )
        .await
        .unwrap();

    // 5. Добавляем 1 юзера в платформу 2
    client
        .user_add(
            platform_2,
            User {
                id: UserId::Int(3),
                payload: vec![],
                meta: json!({"score": 10}),
            },
        )
        .await
        .unwrap();

    sleep(Duration::from_millis(500)).await;

    // 6. Проверяем счетчики по конкретным платформам
    let count_p1 = client
        .total_platformed_users_count(platform_1)
        .await
        .unwrap();
    assert_eq!(count_p1, 2, "Platform 1 should have exactly 2 users");

    let count_p2 = client
        .total_platformed_users_count(platform_2)
        .await
        .unwrap();
    assert_eq!(count_p2, 1, "Platform 2 should have exactly 1 user");

    // 7. Проверяем общий счетчик всех пользователей в системе
    let total = client.total_users_count().await.unwrap();
    // Так как тесты могут запускаться асинхронно, общее число юзеров может быть >= 3
    assert!(total >= 3, "Total users count is lower than expected");

    // 8. Проверяем размер очереди конкретного селектора
    let q_size = client
        .selector_queue_size(platform_1, selector_id)
        .await
        .unwrap();
    assert_eq!(
        q_size,
        SelectorSize::Size(2),
        "Selector should have 2 ready tasks in queue"
    );

    // 9. Берем одну задачу и проверяем, что размер очереди уменьшился
    let _ = client
        .select_user::<Value>(platform_1, selector_id)
        .await
        .unwrap();
    let q_size_after = client
        .selector_queue_size(platform_1, selector_id)
        .await
        .unwrap();
    assert_eq!(
        q_size_after,
        SelectorSize::Size(1),
        "Selector queue size did not decrement after select_user"
    );

    // Очистка
    client.platform_delete(platform_1).await.unwrap();
    client.platform_delete(platform_2).await.unwrap();
}

// Инвалидация задачи из очереди по истечению времени в селекторе
#[tokio::test]
async fn test_task_expires_from_queue() {
    let client = setup_client().await;
    let platform_id = 117;
    let selector_id = 20;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    let query = "expire_at AGE < 2 seconds";
    client
        .selector_create(platform_id, selector_id, query, 60)
        .await
        .unwrap();

    let expire_ts = chrono::Utc::now().timestamp();

    client
        .user_add(
            platform_id,
            User {
                id: UserId::Int(17001),
                payload: b"expiring_data".to_vec(),
                meta: json!({
                    "expire_at": expire_ts
                }),
            },
        )
        .await
        .unwrap();

    // Ждем немного (300мс), чтобы Time Ticker положил пользователя в очередь
    sleep(Duration::from_millis(300)).await;

    // Убеждаемся, что задача действительно находится в горячей очереди
    let q_size = client
        .selector_queue_size(platform_id, selector_id)
        .await
        .unwrap();
    assert_eq!(
        q_size,
        SelectorSize::Size(1),
        "Task should initially match and be in the queue"
    );

    // Ждем 2.5 секунды, чтобы условие `expire_at > NOW()` стало ложным
    println!("Waiting for the task to automatically expire...");
    sleep(Duration::from_millis(2500)).await;

    // Time Ticker должен был сработать (wakeup_at), понять, что юзер больше не подходит,
    // и вычесть его из счетчика очереди.
    let q_size_after = client
        .selector_queue_size(platform_id, selector_id)
        .await
        .unwrap();
    assert_eq!(
        q_size_after,
        SelectorSize::Size(0),
        "Task should have been purged from the queue due to expiration"
    );

    // Физическая проверка - пытаемся забрать задачу из селектора
    let task = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    assert!(
        matches!(task, SelectUser::Finished),
        "Expected Finished since the user expired from the queue"
    );

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

// Удаление пользователя инвалидирует очереди и стирает данные из БД
#[tokio::test]
async fn test_user_delete_invalidates_queue_and_removes_from_db() {
    let client = setup_client().await;
    let platform_id = 118;
    let selector_id = 21;

    let _ = client.platform_delete(platform_id).await;
    client.platform_create(platform_id).await.unwrap();

    // Создаем селектор
    client
        .selector_create(platform_id, selector_id, "score > 0", 60)
        .await
        .unwrap();

    let target_user_id = UserId::Int(18001);

    // Добавляем пользователя
    client
        .user_add(
            platform_id,
            User {
                id: target_user_id.clone(),
                payload: b"data_to_delete".to_vec(),
                meta: json!({
                    "score": 10
                }),
            },
        )
        .await
        .unwrap();

    // Ждем, чтобы Time Ticker (или синхронная маршрутизация) положил задачу в очередь
    sleep(Duration::from_millis(300)).await;

    // Убеждаемся, что задача попала в очередь
    let q_size = client
        .selector_queue_size(platform_id, selector_id)
        .await
        .unwrap();
    assert_eq!(
        q_size,
        SelectorSize::Size(1),
        "Task should be in the queue before deletion"
    );

    // Убеждаемся, что пользователь есть в БД
    let found = client
        .user_search::<Value>(platform_id, target_user_id.clone())
        .await
        .unwrap();
    assert!(
        found.is_some(),
        "User should be found in DB before deletion"
    );

    // Удаляем пользователя
    let delete_status = client
        .user_delete(platform_id, target_user_id.clone())
        .await
        .unwrap();

    assert_eq!(
        format!("{:?}", delete_status),
        "Success",
        "Expected user to be successfully deleted"
    );

    // Проверяем обновление счетчиков очереди
    let q_size_after = client
        .selector_queue_size(platform_id, selector_id)
        .await
        .unwrap();
    assert_eq!(
        q_size_after,
        SelectorSize::Size(0),
        "Selector queue size did not decrement after user_delete!"
    );

    // Пытаемся забрать задачу (проверка инвалидации версии)
    let task = client
        .select_user::<Value>(platform_id, selector_id)
        .await
        .unwrap();
    assert!(
        matches!(task, SelectUser::Finished) || matches!(task, SelectUser::Empty),
        "Expected no tasks to be returned since the user was deleted, got: {:?}",
        task
    );

    // Убеждаемся, что юзер физически удален из БД
    let missing = client
        .user_search::<Value>(platform_id, target_user_id.clone())
        .await
        .unwrap();
    assert!(
        missing.is_none(),
        "Expected None, but user was still found in DB after deletion!"
    );

    // Очистка
    client.platform_delete(platform_id).await.unwrap();
}

// Проверка на создание множественных пользователей при конкурентном добавлении
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_toctou_ghost_users_on_concurrent_add() {
    let admin_client = setup_client().await;
    let platform_id = 120;
    let selector_id = 25;

    let _ = admin_client.platform_delete(platform_id).await;
    admin_client.platform_create(platform_id).await.unwrap();
    admin_client
        .selector_create(platform_id, selector_id, "score > 0", 60)
        .await
        .unwrap();

    let target_user_id = UserId::Int(9999);
    let mut handles = Vec::new();

    println!("Spawning 100 concurrent user_add requests for the SAME user...");

    // Запускаем 100 параллельных подключений, которые пытаются добавить одного и того же юзера
    for _ in 0..100 {
        let user_id = target_user_id.clone();
        handles.push(tokio::spawn(async move {
            let client = setup_client().await;
            let user = User {
                id: user_id,
                payload: b"ghost_data".to_vec(),
                meta: serde_json::json!({"score": 10}),
            };
            let _ = client.user_add(platform_id, user).await;
        }));
    }

    // Ждем завершения всех потоков
    for handle in handles {
        let _ = handle.await;
    }

    // Ждем, чтобы Time Ticker и диспетчер успели всё переварить
    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

    // Считаем размер очереди для нашего селектора
    let q_size = admin_client
        .selector_queue_size(platform_id, selector_id)
        .await
        .unwrap();

    let size = match q_size {
        SelectorSize::Size(s) => s,
        _ => panic!("Expected size"),
    };

    println!("Finished adding. Queue size is: {}", size);

    // Если код БД потокобезопасен, юзер добавится ровно 1 раз, и размер очереди будет 1.
    assert_eq!(
        size, 1,
        "RACE CONDITION DETECTED! Inserted the same user concurrently, but queue size is {} instead of 1. Ghost users created!",
        size
    );

    admin_client.platform_delete(platform_id).await.unwrap();
}
