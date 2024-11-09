use crate::{cipher::magma::{Magma, ID_TC26_GOST_28147_PARAM_Z}, tests::cipher::magma::common::create_test_cipher};

#[test]
fn test_key_generation() {
    let cipher = create_test_cipher();
    let round_keys = cipher.get_round_keys();

    let expected = [
        0xffeeddcc, 0xbbaa9988, 0x77665544, 0x33221100,
        0xf0f1f2f3, 0xf4f5f6f7, 0xf8f9fafb, 0xfcfdfeff,
        0xffeeddcc, 0xbbaa9988, 0x77665544, 0x33221100,
        0xf0f1f2f3, 0xf4f5f6f7, 0xf8f9fafb, 0xfcfdfeff,
        0xffeeddcc, 0xbbaa9988, 0x77665544, 0x33221100,
        0xf0f1f2f3, 0xf4f5f6f7, 0xf8f9fafb, 0xfcfdfeff,
        0xfcfdfeff, 0xf8f9fafb, 0xf4f5f6f7, 0xf0f1f2f3,
        0x33221100, 0x77665544, 0xbbaa9988, 0xffeeddcc,
    ];

    assert_eq!(round_keys.len(), 32, "Неверное количество раундовых ключей");

    assert_eq!(round_keys, expected);
}

#[test]
#[should_panic]
fn test_invalid_key_lenght() {
    let invalid_key = vec![0u8; 31];
    Magma::new(&invalid_key, ID_TC26_GOST_28147_PARAM_Z).unwrap();
}

#[test]
#[should_panic]
fn test_invalid_zeroed_key() {
    let invalid_key = vec![0u8; 32];
    Magma::new(&invalid_key, ID_TC26_GOST_28147_PARAM_Z).unwrap();
}
