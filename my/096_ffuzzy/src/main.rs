use ssdeep::{Generator, RawFuzzyHash};

////////////////////////////////////////////////////////////////////////////////////////////

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

////////////////////////////////////////////////////////////////////////////////////////////

fn test_2() {
    // Данные для подсчета хешей
    let buf1: &[u8] = b"Hello, ";
    let buf2: &[u8; 6] = b"World!";

    // Непосредственно сам генератор для накопления хешей
    let mut generator = Generator::new();

    // Опционально, но желательно указать размер входных будущих данных,
    // так как это улучшает производительность.
    generator
        .set_fixed_input_size_in_usize(buf1.len() + buf2.len() + 1)
        .unwrap();

    // Добавляем теперь в расчет хешей эти самые данные
    generator.update(buf1);
    generator.update_by_iter((*buf2).into_iter());
    generator.update_by_byte(b'\n');

    // Получаем теперь хеш
    let hash: RawFuzzyHash = generator.finalize().unwrap();

    // Конвертируем хеш в строку и сравниваем
    assert_eq!(hash.to_string(), "3:aaX8v:aV");
}

fn test_3() {
    // Requires either the "alloc" feature or std environment on your crate
    // to use the `to_string()` method (default enabled).
    use ssdeep::{FuzzyHash, FuzzyHashCompareTarget};

    // Those fuzzy hash strings are "normalized" so that easier to compare.
    let str1 = "12288:+ySwl5P+C5IxJ845HYV5sxOH/cccccccei:+Klhav84a5sxJ";
    let str2 = "12288:+yUwldx+C5IxJ845HYV5sxOH/cccccccex:+glvav84a5sxK";
    // FuzzyHash object can be used to avoid parser / normalization overhead
    // and helps improving the performance.
    let hash1: FuzzyHash = str::parse(str1).unwrap();
    let hash2: FuzzyHash = str::parse(str2).unwrap();

    // Note that converting the (normalized) fuzzy hash object back to the string
    // may not preserve the original string.  To preserve the original fuzzy hash
    // string too, consider using dual fuzzy hashes (such like DualFuzzyHash) that
    // preserves the original string in the compressed format.
    // *   str1:  "12288:+ySwl5P+C5IxJ845HYV5sxOH/cccccccei:+Klhav84a5sxJ"
    // *   hash1: "12288:+ySwl5P+C5IxJ845HYV5sxOH/cccei:+Klhav84a5sxJ"
    assert_ne!(hash1.to_string(), str1);

    // If we have number of fuzzy hashes and a hash is compared more than once,
    // storing those hashes as FuzzyHash objects is faster.
    assert_eq!(hash1.compare(&hash2), 88);

    // But there's another way of comparison.
    // If you compare "a fuzzy hash" with "other many fuzzy hashes", this method
    // (using FuzzyHashCompareTarget as "a fuzzy hash") is much, much faster.
    let target: FuzzyHashCompareTarget = FuzzyHashCompareTarget::from(&hash1);
    assert_eq!(target.compare(&hash2), 88);

    // If you reuse the same `target` object repeatedly for multiple fuzzy hashes,
    // `new()` and `init_from()` will be helpful.
    let mut target: FuzzyHashCompareTarget = FuzzyHashCompareTarget::new();
    target.init_from(&hash1);
    assert_eq!(target.compare(&hash2), 88);
}

fn main() {
    test_1();
    test_2();
}
