use rusqlite::{Connection, ToSql, Transaction};

use sqlite3_insert_benchmarks as common;

fn faker_wrapper(mut conn: Connection, count: i64) {
    // Запуск транзакции
    let tx = conn.transaction().unwrap();
    // Работа
    faker(&tx, count);
    // Коммитим транзакцию
    tx.commit().unwrap();
}

fn faker(tx: &Transaction, count: i64) {
    // Размер группы для пакетного добавления
    let min_batch_size: i64 = 50;
    if count < min_batch_size {
        panic!("count cant be less than min batch size");
    }

    // Повторяем 50 раз строку для параметов
    let mut stmt_with_area = {
        let mut with_area_params = " (NULL, ?, ?, ?),".repeat(min_batch_size as usize);
        with_area_params.pop();
        let with_area_params = with_area_params.as_str();

        let st = format!("INSERT INTO user VALUES {}", with_area_params);

        // Кешируем запрос
        tx.prepare_cached(st.as_str()).unwrap()
    };

    // Повторяем 50 раз строку для параметов
    let mut stmt = {
        let mut without_area_params = " (NULL, NULL, ?, ?),".repeat(min_batch_size as usize);
        without_area_params.pop();
        let without_area_params = without_area_params.as_str();

        let st = format!("INSERT INTO user VALUES {}", without_area_params);

        // Кешируем запрос
        tx.prepare_cached(st.as_str()).unwrap()
    };

    for _ in 0..(count / min_batch_size) {
        // Рандомные параметры
        let with_area = common::get_random_bool();
        let age = common::get_random_age();
        let is_active = common::get_random_active();

        let mut param_values: Vec<_> = Vec::new();
        
        if with_area {
            // lets prepare the batch
            let mut vector = Vec::<(String, i8, i8)>::new();
            for _ in 0..min_batch_size {
                let area_code = common::get_random_area_code();
                vector.push((area_code, age, is_active));
            }
            for batch in vector.iter() {
                param_values.push(&batch.0 as &dyn ToSql);
                param_values.push(&batch.1 as &dyn ToSql);
                param_values.push(&batch.2 as &dyn ToSql);
            }
            stmt_with_area.execute(&*param_values).unwrap();
        } else {
            // lets prepare the batch
            let mut vector = Vec::<(i8, i8)>::new();
            for _ in 0..min_batch_size {
                vector.push((age, is_active));
            }
            for batch in vector.iter() {
                param_values.push(&batch.0 as &dyn ToSql);
                param_values.push(&batch.1 as &dyn ToSql);
            }
            stmt.execute(&*param_values).unwrap();
        }
    }
}

fn main() {
    let conn = Connection::open("basic_batched.db").unwrap();
    conn.execute_batch(common::pragma_rules()).expect("PRAGMA");
    conn.execute(
        "CREATE TABLE IF NOT EXISTS user (
                id INTEGER not null primary key,
                area CHAR(6),
                age INTEGER not null,
                active INTEGER not null)",
        [],
    )
    .unwrap();
    faker_wrapper(conn, 1_000_000)
}