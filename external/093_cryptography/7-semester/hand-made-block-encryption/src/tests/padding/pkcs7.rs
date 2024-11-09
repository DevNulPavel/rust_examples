use crate::padding::PKCS7;

#[test]
fn test_pad() {
    let padding = PKCS7;
    let block_size = 8;

    let test_cases = vec![
        // Пустые данные
        (
            vec![],
            vec![8, 8, 8, 8, 8, 8, 8, 8],
        ),
        // Данные длиной в 1 байт
        (
            vec![1],
            vec![1, 7, 7, 7, 7, 7, 7, 7],
        ),
        // Данные длиной в 7 байт
        (
            vec![1, 2, 3, 4, 5, 6, 7],
            vec![1, 2, 3, 4, 5, 6, 7, 1],
        ),
        // Данные длиной в блок
        (
            vec![1, 2, 3, 4, 5, 6, 7, 8],
            vec![1, 2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8],
        ),
        // Данные длиной больше блока
        (
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 7, 7, 7, 7, 7, 7, 7],
        ),
    ];

    for (input, expected) in test_cases {
        let result = padding.test_pad(&input, block_size).unwrap();
        assert_eq!(result, expected, "Ошибка при добавлении заполнения");
        assert_eq!(result.len() % block_size, 0, "Размер результата не кратен размеру блока");
    }
}

#[test]
fn test_unpad() {
    let padding = PKCS7;

    let test_cases = vec![
        // Один блок заполнения
        (
            vec![8, 8, 8, 8, 8, 8, 8, 8],
            vec![],
        ),
        // Заполнение длинной в 7 байт
        (
            vec![1, 7, 7, 7, 7, 7, 7, 7],
            vec![1],
        ),
        // Заполнение длинной в 1 блок
        (
            vec![1, 2, 3, 4, 5, 6, 7, 8, 8, 8, 8, 8, 8, 8, 8, 8],
            vec![1, 2, 3, 4, 5, 6, 7, 8],
        ),
        // Заполнение длинной в 1 байт
        (
            vec![1, 2, 3, 4, 5, 6, 7, 1],
            vec![1, 2, 3, 4, 5, 6, 7],
        ),
    ];

    for (input, expected) in test_cases {
        let result = padding.test_unpad(&input).unwrap();
        assert_eq!(result, expected, "Ошибка при удалении заполнения");
    }
}

#[test]
fn test_invalid_block_size() {
    let padding = PKCS7;
    let data = vec![1, 2, 3, 4];

    // Нулевой размер блока
    assert!(padding.test_pad(&data, 0).is_err());

    // Слишком большой размер блока
    assert!(padding.test_pad(&data, 256).is_err());
}

#[test]
fn test_invalid_padding() {
    let padding = PKCS7;

    let test_cases = vec![
        // Пустые данные
        vec![],
        // Некорректное значение заполнения
        vec![1, 2, 3, 4, 5],
        // Неверное заполнение
        vec![1, 2, 3, 4, 3, 3, 3, 2],
    ];

    for input in test_cases {
        assert!(padding.test_unpad(&input).is_err());
    }
}
