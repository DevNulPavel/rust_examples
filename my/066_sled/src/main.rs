use std::str::from_utf8;

macro_rules! utf8_parse {
    ($val: expr, $err_message: literal) => {
        from_utf8($val).expect($err_message)
    };
}

fn main() {
    // Красивые сообщения о панике
    human_panic::setup_panic!();

    // Open db with default config
    let database = sled::open("./db/test_database.sled").expect("Sled open failed");

    // Insert value
    database.insert(b"test_key_1", b"test_value_1").expect("Insert failed");
    database.insert(b"test_key_2", b"test_value_2").expect("Insert failed");
    database.insert(b"test_key_3", b"test_value_3").expect("Insert failed");

    // Receive value
    let result = database
        .get(b"test_key_1")
        .expect("Receive failed")
        .expect("Value by key is missing");
    println!("Received value: {:#?}", utf8_parse!(&result, "Utf8 parse failed"));

    // Compare and swap
    // Если старое значение None, значит просто устанавливает новое значение если старого нету
    // Если новое значение None, значит просто удаляем значение, если совпадает со старым
    database
        .compare_and_swap(b"test_key_1", Some(b"test_value_1"), Some(b"test_value_1_new"))
        .expect("Sled error")
        .expect("Compare and swap error");

    // Итерируемся по ключам, сортировка идет побайтово
    database.range(b"test_key_1".as_ref()..).filter_map(|v| v.ok()).for_each(|v| {
        let key = utf8_parse!(v.0.as_ref(), "Key parse failed");
        let value = utf8_parse!(v.1.as_ref(), "Value parse failed");
        println!("{}: {}", key, value);
    });

    // Удаление значения
    database.remove(b"test_key_3").expect("Field remove success");
}
