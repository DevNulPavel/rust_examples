use super::common::create_test_cipher;

#[test]
fn test_transform_t() {
    let cipher = create_test_cipher();

    let test_cases = vec![
        (0xfdb97531, 0x2a196f34),
        (0x2a196f34, 0xebd9f03a),
        (0xebd9f03a, 0xb039bb3d),
        (0xb039bb3d, 0x68695433),
    ];

    for (input, expected) in test_cases {
        let result = cipher.test_transform_t(input);
        assert_eq!(
            result,
            expected,
            "Ошибка при вводе: {:08x}, ожидалось: {:08x}, а в результате: {:08x}",
            input,
            expected,
            result,
        );
    }
}
