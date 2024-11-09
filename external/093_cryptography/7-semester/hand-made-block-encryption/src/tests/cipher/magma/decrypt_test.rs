use crate::tests::cipher::magma::common::create_test_cipher;

#[test]
fn test_decryption() {
    let cipher = create_test_cipher();

    let input = [0x4e, 0xe9, 0x01, 0xe5, 0xc2, 0xd8, 0xca, 0x3d];
    let expected = [0xfe, 0xdc, 0xba, 0x98, 0x76, 0x54, 0x32, 0x10];

    let decrypted_block = cipher.test_decrypt_block(&input).unwrap();

    assert_eq!(decrypted_block.len(), 8, "Неверный размер блока текста");

    assert_eq!(decrypted_block, expected);
}
