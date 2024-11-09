use chrono::Utc;
use hand_made_el_gamal::ElGamal;
use hand_made_fiat_shamir::FiatShamir;
use hand_made_rsa::keys::{PrivateKey, PublicKey};
use hand_made_rsa::{TypeKey, RSA};
use hand_made_sha::{sha256, sha512};
use hand_made_streebog::{streebog_256, streebog_512};
use hash_function_wrapper::HashFunc;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use pkcs7::{SetOfAttributeValue, Timestamp, PKCS7};

fn main() {
    println!("🟡 Создание сервера ...");
    let tcp_listener = TcpListener::bind("127.0.0.1:8090").expect("🔴 Ошибка");
    println!("🟢 Сервер успешно создан");

    let file_name = "pkcs7_server.json";
    loop {
        if let Ok((mut stream, _)) = tcp_listener.accept() {
            println!("🔽 Соединение установлено");

            let pkcs7 = validate_signature(&stream);
            pkcs7.save_to_json(file_name);
            println!("🔼 Данные готовы к отправлению");

            stream
                .write(serde_json::to_string(&pkcs7).unwrap().as_bytes())
                .unwrap();
            println!("🔼 Данные отправлены");
        }
    }
}

/// # Функция проверки подписи клиента
fn validate_signature(mut stream: &TcpStream) -> PKCS7 {
    let mut buffer = [0; 100000];
    stream.read(&mut buffer).unwrap();

    let mut buffer = buffer.to_vec();
    buffer.retain(|e| e != &0);

    let mut pkcs7: PKCS7 = serde_json::from_str(&String::from_utf8_lossy(&buffer)).unwrap();

    // Получаем все необходимые данные для дальнейшей проверки подписи
    let signature_algorithm_identifier = pkcs7.SignerInfos.SignatureAlgorithmIdentifier.clone();
    let message = pkcs7.EncapsulatedContentInfo.OCTET_STRING_OPTIONAL.clone();
    let digest_algorithm_identifier = pkcs7.DigestAlgorithmIdentifiers.clone();
    let user_open_key = pkcs7.SignerInfos.SubjectPublicKeyInfo.clone();
    let user_signature = pkcs7.SignerInfos.SignatureValue.clone();

    // Формируем новое сообщение для хеширования
    let new_message = message.clone() + user_signature.as_str();

    // Получаем хеш от изначального сообщения (для проверки) и нового сообщения (для формирования подписи)
    let (hash_user_message, hash_new_message) = match digest_algorithm_identifier.as_str() {
        "SHA256" => {
            println!("🔀 Функция хеширования SHA256");
            (
                sha256(message.as_bytes())
                    .iter()
                    .map(|e| format!("{:02x}", e))
                    .collect::<String>(),
                sha256(new_message.as_bytes())
                    .iter()
                    .map(|e| format!("{:02x}", e))
                    .collect::<String>(),
            )
        }

        "SHA512" => {
            println!("🔀 Функция хеширования SHA512");
            (
                sha512(message.as_bytes())
                    .iter()
                    .map(|e| format!("{:02x}", e))
                    .collect::<String>(),
                sha512(new_message.as_bytes())
                    .iter()
                    .map(|e| format!("{:02x}", e))
                    .collect::<String>(),
            )
        }

        "STREEBOG256" => {
            println!("🔀 Функция хеширования STREEBOG256");
            (
                streebog_256(message.as_bytes())
                    .iter()
                    .map(|e| format!("{:02x}", e))
                    .collect::<String>(),
                streebog_256(new_message.as_bytes())
                    .iter()
                    .map(|e| format!("{:02x}", e))
                    .collect::<String>(),
            )
        }

        "STREEBOG512" => {
            println!("🔀 Функция хеширования STREEBOG512");
            (
                streebog_512(message.as_bytes())
                    .iter()
                    .map(|el| format!("{:02x}", el))
                    .collect::<String>(),
                streebog_512(new_message.as_bytes())
                    .iter()
                    .map(|el| format!("{:02x}", el))
                    .collect::<String>(),
            )
        }

        _ => ("".to_string(), "".to_string()),
    };

    match signature_algorithm_identifier.as_str() {
        "RSA" => {
            println!("🔀 Функция шифрования RSA");
            let rsa_for_validate = RSA::create(
                PrivateKey::from_raw_parts("0", "0", "0"),
                PublicKey::create_key_from_hashmap(user_open_key),
            );
            let hash_for_verification =
                rsa_for_validate.decrypt_msg(TypeKey::PublicKey, &user_signature);

            if hash_for_verification == hash_user_message {
                println!("🟢 Проверка выполнена успешно");
                let time_stamp = create_time_stamp();
                let rsa = RSA::generate_keys();
                let server_signature = rsa.encrypt_msg(
                    TypeKey::PrivateKey,
                    &(hash_new_message.clone() + time_stamp.0.as_str()),
                );

                pkcs7.SignerInfos.UnsignedAttributes.SetOfAttributeValue =
                    Some(SetOfAttributeValue {
                        Hash: hash_new_message,
                        Timestamp: Timestamp {
                            UTCTime: time_stamp.0,
                            GeneralizedTime: time_stamp.1,
                        },
                        Signature: server_signature,
                        Certificate: rsa.get_public_key(),
                    })
            }
        }

        "ElGamal" => {
            println!("🔀 Функция шифрования Эль-Гамаль");
            let (y, delta) = user_signature.split_once("|").unwrap();
            let public_key = hand_made_el_gamal::PublicKey::from_hashmap(user_open_key);
            let mut el_gamal = ElGamal::default();
            el_gamal.set_public_key(public_key);

            if el_gamal.check_signature((y.to_string(), delta.to_string()), &hash_user_message) {
                println!("🟢 Проверка выполнена успешно");
                let time_stamp = create_time_stamp();
                let el_gamal = ElGamal::generate_system();
                let server_signature = el_gamal
                    .sign_message((hash_new_message.to_string() + time_stamp.0.as_str()).as_str());
                let server_signature = server_signature.0 + "|" + server_signature.1.as_str();

                pkcs7.SignerInfos.UnsignedAttributes.SetOfAttributeValue =
                    Some(SetOfAttributeValue {
                        Hash: hash_new_message,
                        Timestamp: Timestamp {
                            UTCTime: time_stamp.0,
                            GeneralizedTime: time_stamp.1,
                        },
                        Signature: server_signature,
                        Certificate: el_gamal.get_public_key(),
                    })
            }
        }

        "FiatShamir" => {
            println!("🔀 Функция шифрования Фиат-Шамир");
            let hash_func = match digest_algorithm_identifier.as_str() {
                "SHA256" => HashFunc::Sha256,
                "SHA512" => HashFunc::Sha512,
                "STREEBOG256" => HashFunc::Streebog256,
                "STREEBOG512" => HashFunc::Streebog512,
                _ => panic!(),
            };
            let public_key = hand_made_fiat_shamir::PublicKey::from_hashmap(user_open_key);
            let mut fiat_shamir = FiatShamir::default();
            fiat_shamir.set_public_key(public_key);
            fiat_shamir.set_hash_func(hash_func.clone());

            dbg!(&fiat_shamir);
            dbg!(&user_signature);
            dbg!(&message);

            if fiat_shamir.check_signature(user_signature, message) {
                println!("🟢 Проверка выполнена успешно");
                let time_stamp = create_time_stamp();
                let fiat_shamir = FiatShamir::generate_system(&hash_func);
                let server_signature =
                    fiat_shamir.sign_message((new_message + time_stamp.0.as_str()).as_str());

                pkcs7.SignerInfos.UnsignedAttributes.SetOfAttributeValue =
                    Some(SetOfAttributeValue {
                        Hash: hash_new_message,
                        Timestamp: Timestamp {
                            UTCTime: time_stamp.0,
                            GeneralizedTime: time_stamp.1,
                        },
                        Signature: server_signature,
                        Certificate: fiat_shamir.get_public_key(),
                    })
            }
        }

        _ => (),
    };

    pkcs7
}

/// # Функция создания временных штампов
/// - utc_time
/// - generalized_time
fn create_time_stamp() -> (String, String) {
    let utc_now = Utc::now();
    let utc_time_str = utc_now.format("%y%m%d%H%M%SZ").to_string();

    let generalized_time_str = utc_now.format("%Y%m%d%H%M%SZ").to_string();
    (utc_time_str, generalized_time_str)
}
