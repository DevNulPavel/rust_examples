use crate::cipher::kuznyechik::cipher::Kuznyechik;

#[test]
fn test_gf_multiply() {
    let test_cases = vec![
        (10, 20, 136),
        (20, 10, 136),
        (80, 36, 17),
        (36, 80, 17),
        (255, 255, 6),
        (0, 255, 0),
        (1, 2, 2),
        (24, 62, 85),
        (85, 27, 11),
        (90, 12, 62),
    ];

    for (a, b, expected) in test_cases {
        let result = Kuznyechik::test_gf_multiply(a, b);
        assert_eq!(
            result,
            expected,
            "Ошибка при вводе: a: {}, b: {}, ожидалось: {}, а в результате: {}",
            a,
            b,
            expected,
            result,
        );
    }
}
