use crate::tests::cipher::kuznyechik::common::create_test_cipher;

#[test]
fn test_encryption() {
    let cipher = create_test_cipher();

    let input = [0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x00, 0xff, 0xee, 0xdd, 0xcc, 0xbb, 0xaa, 0x99, 0x88];
    let expected = [0x7f, 0x67, 0x9d, 0x90, 0xbe, 0xbc, 0x24, 0x30, 0x5a, 0x46, 0x8d, 0x42, 0xb9, 0xd4, 0xed, 0xcd];

    let encrypted_block = cipher.test_encrypt_block(&input).unwrap();

    assert_eq!(encrypted_block.len(), 16, "Неверный размер блока шифротекста");

    assert_eq!(encrypted_block, expected);
}