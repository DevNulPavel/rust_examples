use super::common::create_test_cipher;

#[test]
fn test_transform_g() {
    let cipher = create_test_cipher();

    let test_cases = vec![
        ((0x87654321, 0xfedcba98), 0xfdcbc20c),
        ((0xfdcbc20c, 0x87654321), 0x7e791a4b),
        ((0x7e791a4b, 0xfdcbc20c), 0xc76549ec),
        ((0xc76549ec, 0x7e791a4b), 0x9791c849),
    ];

    for ((input, key), expected) in test_cases {
        let result = cipher.test_transform_g(input, key);
        assert_eq!(
            result,
            expected,
            "Ошибка при вводе: (input: {:08x}, key: {:08x}), ожидалось: {:08x}, а в результате: {:08x}",
            input,
            key,
            expected,
            result,
        );
    }
}
