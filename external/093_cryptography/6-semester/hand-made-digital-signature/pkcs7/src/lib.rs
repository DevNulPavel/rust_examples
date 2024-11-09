#![allow(non_snake_case)]

use hash_function_wrapper::HashFunction;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

/// # Трейт для использования произвольных шифров в PKCS 7
pub trait EncryptionFunction {
    /// # Возвращает идентификатор шифра
    fn get_id(&self) -> String;

    /// # Получение публичного ключа
    fn get_public_key(&self) -> HashMap<String, String>;

    /// # Шифрование данных
    fn encrypt(&self, message: &String) -> String;
}

/// # Стандарт PKCS 7
/// - `CMSVersion` - версия используемого синтаксиса
/// - `DigestAlgorithmIdentifiers` - идентификатор алгоритма хеширования
/// - `EncapsulatedContentInfo` - встраиваемая информация о подписываемом контенте
/// - `SignerInfos` - информация о том, кто подписывает документ
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PKCS7 {
    CMSVersion: String,
    pub DigestAlgorithmIdentifiers: String,
    pub EncapsulatedContentInfo: EncapsulatedContentInfo,
    pub SignerInfos: SignerInfos,
}

/// # Встраиваемая информация о подписываемом контенте
/// - `ContentType` - вид подписываемой информации
/// - `OCTET_STRING_OPTIONAL` - опциональная строка данных
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct EncapsulatedContentInfo {
    pub ContentType: String,
    pub OCTET_STRING_OPTIONAL: String,
}

/// # Структура цифровой подписи
/// - `CMSVersion` - версия используемого синтаксиса
/// - `SignerIdentifier` - идентификатор автора
/// - `DigestAlgorithmIdentifier` - идентификатор алгоритма хеширования
/// - `SignatureAlgorithmIdentifier` - идентификатора алгоритма подписи
/// - `SignatureValue` - значение цифровой подписи в шестнадцатеричном виде
/// - `SubjectPublicKeyInfo` - информация об открытом ключе
/// - `UnsignedAttributes` - опциональное поле неподписанных данных
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SignerInfos {
    CMSVersion: String,
    pub SignerIdentifier: String,
    pub DigestAlgorithmIdentifier: String,
    pub SignatureAlgorithmIdentifier: String,
    pub SignatureValue: String,
    pub SubjectPublicKeyInfo: HashMap<String, String>,
    pub UnsignedAttributes: UnsignedAttributes,
}

/// # Поле неподписанных данных
/// - `OBJECT_IDENTIFIER` - идентификатор объекта
/// - `SetOfAttributeValue` - Набор атрибутов
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnsignedAttributes {
    pub OBJECT_IDENTIFIER: String,
    pub SetOfAttributeValue: Option<SetOfAttributeValue>,
}

/// # Поля подписи центра штампов времени
/// - `Hash` - хэш
/// - `Timestamp` - штамп времени
/// - `Signature` - сигнатура центра штампов времени
/// - `Certificate` - сертификат центра штампов времени
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SetOfAttributeValue {
    pub Hash: String,
    pub Timestamp: Timestamp,
    pub Signature: String,
    pub Certificate: HashMap<String, String>,
}

/// # Штамп времени
/// - `UTCTime` - время по Гринвичу
/// - `GeneralizedTime` - время центра штампов времени
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Timestamp {
    pub UTCTime: String,
    pub GeneralizedTime: String,
}

impl PKCS7 {
    /// # Конструктор PKCS 7
    pub fn new_pkcs7(
        encryption_function: &dyn EncryptionFunction,
        hash_function: &dyn HashFunction,
        message: &String,
        signer_identifier: String,
    ) -> PKCS7 {
        let signature: String;
        if encryption_function.get_id() == "FiatShamir".to_string() {
            signature = encryption_function.encrypt(&message);
        } else {
            signature = encryption_function.encrypt(&hash_function.hash(&message))
        }

        let pkcs7 = PKCS7 {
            CMSVersion: "1".to_string(),
            DigestAlgorithmIdentifiers: hash_function.get_id(),
            EncapsulatedContentInfo: EncapsulatedContentInfo {
                ContentType: "text".to_string(),
                OCTET_STRING_OPTIONAL: message.clone(),
            },
            SignerInfos: SignerInfos {
                CMSVersion: "1".to_string(),
                SignerIdentifier: signer_identifier,
                DigestAlgorithmIdentifier: hash_function.get_id(),
                SignatureAlgorithmIdentifier: encryption_function.get_id(),
                SignatureValue: signature,
                SubjectPublicKeyInfo: encryption_function.get_public_key(),
                UnsignedAttributes: UnsignedAttributes {
                    OBJECT_IDENTIFIER: "1".to_string(),
                    SetOfAttributeValue: None,
                },
            },
        };

        pkcs7
    }

    /// # Сохраняет PKCS 7 в json файл
    pub fn save_to_json(&self, file_name: &str) {
        let mut file = File::create(file_name).unwrap();
        let data = serde_json::to_string_pretty(self).unwrap();
        file.write_all(data.as_bytes()).unwrap();
    }
}
