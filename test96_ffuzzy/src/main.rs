fn test_1() {
    // Считаем хеш от файлика
    let hash_value_1 = ssdeep::hash_buf(include_bytes!("../data_samples/file_1")).unwrap();
    let hash_value_1_string = hash_value_1.to_string();
    println!("File 1 hash: {}", hash_value_1_string);

    // Считаем хеш от файлика
    let hash_value_2 = ssdeep::hash_buf(include_bytes!("../data_samples/file_2")).unwrap();
    let hash_value_2_string = hash_value_2.to_string();
    println!("File 2 hash: {}", hash_value_2_string);

    // Сравнение двух полученных хешей
    let score =
        ssdeep::compare(hash_value_1_string.as_str(), hash_value_2_string.as_str()).unwrap();
    assert_eq!(score, 8);
}

fn main() {
    test_1();
}
