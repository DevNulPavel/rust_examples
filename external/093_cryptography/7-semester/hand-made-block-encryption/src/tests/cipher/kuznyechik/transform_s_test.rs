use crate::cipher::kuznyechik::cipher::Kuznyechik;

#[test]
fn test_transform_s() {
    let test_cases = vec![
        (
            [0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x00],
            [0xb6, 0x6c, 0xd8, 0x88, 0x7d, 0x38, 0xe8, 0xd7, 0x77, 0x65, 0xae, 0xea, 0x0c, 0x9a, 0x7e, 0xfc],
        ),
        (
            [0xb6, 0x6c, 0xd8, 0x88, 0x7d, 0x38, 0xe8, 0xd7, 0x77, 0x65, 0xae, 0xea, 0x0c, 0x9a, 0x7e, 0xfc],
            [0x55, 0x9d, 0x8d, 0xd7, 0xbd, 0x06, 0xcb, 0xfe, 0x7e, 0x7b, 0x26, 0x25, 0x23, 0x28, 0x0d, 0x39],
        ),
        (
            [0x55, 0x9d, 0x8d, 0xd7, 0xbd, 0x06, 0xcb, 0xfe, 0x7e, 0x7b, 0x26, 0x25, 0x23, 0x28, 0x0d, 0x39],
            [0x0c, 0x33, 0x22, 0xfe, 0xd5, 0x31, 0xe4, 0x63, 0x0d, 0x80, 0xef, 0x5c, 0x5a, 0x81, 0xc5, 0x0b],
        ),
        (
            [0x0c, 0x33, 0x22, 0xfe, 0xd5, 0x31, 0xe4, 0x63, 0x0d, 0x80, 0xef, 0x5c, 0x5a, 0x81, 0xc5, 0x0b],
            [0x23, 0xae, 0x65, 0x63, 0x3f, 0x84, 0x2d, 0x29, 0xc5, 0xdf, 0x52, 0x9c, 0x13, 0xf5, 0xac, 0xda],
        ),
    ];

    for (input, expected) in test_cases {
        let mut result = input;
        Kuznyechik::test_transform_s(&mut result);
        assert_eq!(
            result,
            expected,
            "Ошибка при вводе: {:?}, ожидалось: {:?}, а в результате: {:?}",
            input,
            expected,
            result,
        );
    }
}

#[test]
fn test_transform_s_rev() {
    let test_cases = vec![
        (
            [0xb6, 0x6c, 0xd8, 0x88, 0x7d, 0x38, 0xe8, 0xd7, 0x77, 0x65, 0xae, 0xea, 0x0c, 0x9a, 0x7e, 0xfc],
            [0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x00],
        ),
        (
            [0x55, 0x9d, 0x8d, 0xd7, 0xbd, 0x06, 0xcb, 0xfe, 0x7e, 0x7b, 0x26, 0x25, 0x23, 0x28, 0x0d, 0x39],
            [0xb6, 0x6c, 0xd8, 0x88, 0x7d, 0x38, 0xe8, 0xd7, 0x77, 0x65, 0xae, 0xea, 0x0c, 0x9a, 0x7e, 0xfc],
        ),
        (
            [0x0c, 0x33, 0x22, 0xfe, 0xd5, 0x31, 0xe4, 0x63, 0x0d, 0x80, 0xef, 0x5c, 0x5a, 0x81, 0xc5, 0x0b],
            [0x55, 0x9d, 0x8d, 0xd7, 0xbd, 0x06, 0xcb, 0xfe, 0x7e, 0x7b, 0x26, 0x25, 0x23, 0x28, 0x0d, 0x39],
        ),
        (
            [0x23, 0xae, 0x65, 0x63, 0x3f, 0x84, 0x2d, 0x29, 0xc5, 0xdf, 0x52, 0x9c, 0x13, 0xf5, 0xac, 0xda],
            [0x0c, 0x33, 0x22, 0xfe, 0xd5, 0x31, 0xe4, 0x63, 0x0d, 0x80, 0xef, 0x5c, 0x5a, 0x81, 0xc5, 0x0b],
        ),
    ];

    for (input, expected) in test_cases {
        let mut result = input;
        Kuznyechik::test_transform_s_rev(&mut result);
        assert_eq!(
            result,
            expected,
            "Ошибка при вводе: {:?}, ожидалось: {:?}, а в результате: {:?}",
            input,
            expected,
            result,
        );
    }
}