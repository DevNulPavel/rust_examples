use hand_made_el_gamal::ElGamal;
use hand_made_fiat_shamir::FiatShamir;
use hand_made_rsa::keys::{PrivateKey, PublicKey};
use hand_made_rsa::{TypeKey, RSA};
use hand_made_sha::{sha256, sha512};
use hand_made_streebog::{streebog_256, streebog_512};
use hash_function_wrapper::{HashFunc, HashFunction};
use implementation::{ElGamalWrapper, Rsa};
use pkcs7::{EncryptionFunction, PKCS7};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Menu {
    Main,
    Additional,
    ChooseHash,
    MenuEnd,
}

pub struct DataInput {
    hash_func: Option<Box<dyn HashFunction>>,
    encryption_system: Option<Box<dyn EncryptionFunction>>,
}

impl DataInput {
    pub fn get_data(self) -> (Box<dyn HashFunction>, Box<dyn EncryptionFunction>) {
        (self.hash_func.unwrap(), self.encryption_system.unwrap())
    }
}

impl Default for DataInput {
    fn default() -> Self {
        DataInput {
            hash_func: None,
            encryption_system: None,
        }
    }
}

/// Функция обрабатывающая ввод пользователя
pub fn get_data_from_menu(
    menu_type: Menu,
    data_input: &mut DataInput,
) -> Result<Option<Menu>, Box<dyn Error>> {
    let mut input = String::new();

    match menu_type {
        Menu::Main => {
            println!("1 - Зашифровать и отправить на подпись\n2 - Проверить подпись ЦШВ");
            std::io::stdin().read_line(&mut input).unwrap();
            return match input.trim().parse::<usize>()? {
                1 => Ok(Some(Menu::ChooseHash)),
                2 => {
                    validate_signature();
                    Ok(None)
                }
                _ => Err("Недопустимое значение".into()),
            };
        }

        Menu::ChooseHash => {
            println!("Выберите хеш функцию:\n1 - sha256\n2 - sha512\n3 - GOST256\n4 - GOST512");
            std::io::stdin().read_line(&mut input).unwrap();

            match input.trim().parse::<usize>()? {
                1 => data_input.hash_func = Some(Box::new(HashFunc::Sha256)),
                2 => data_input.hash_func = Some(Box::new(HashFunc::Sha512)),
                3 => data_input.hash_func = Some(Box::new(HashFunc::Streebog256)),
                4 => data_input.hash_func = Some(Box::new(HashFunc::Streebog512)),
                _ => return Err("Недопустимое значение".into()),
            };
            Ok(Some(Menu::Additional))
        }

        Menu::Additional => {
            println!("Выберите систему шифрования:\n1 - RSAdsi\n2 - DSAdsi\n3 - FiataShamilya");
            std::io::stdin().read_line(&mut input).unwrap();

            match input.trim().parse::<usize>()? {
                1 => data_input.encryption_system = Some(Box::new(Rsa::generate_keys())),
                2 => {
                    data_input.encryption_system = Some(Box::new(ElGamalWrapper::generate_system()))
                }
                3 => {
                    data_input.encryption_system = Some(Box::new(
                        hand_made_fiat_shamir::FiatShamir::generate_system(data_input.hash_func.as_ref().unwrap().as_ref()),
                    ))
                }
                _ => return Err("Недопустимое значение".into()),
            }
            Ok(Some(Menu::MenuEnd))
        }

        Menu::MenuEnd => Ok(None),
    }
}

/// Функция отправления pkcs7 документа
pub fn send_file(mut tcp_stream: &TcpStream, file_name: &str) {
    let mut file = File::open(file_name).expect("File not found");
    let mut file_data = String::new();

    file.read_to_string(&mut file_data).unwrap();

    dbg!(&file_data);

    tcp_stream.write_all(file_data.as_bytes()).unwrap();
}

/// Функция проверяющая на стороне клиента подпись ЦШВ
pub fn validate_signature() {
    // получаем данные с файла, который прислал сервер
    let mut file = File::open("pkcs7_server.json").unwrap();
    let mut input = String::new();
    file.read_to_string(&mut input).unwrap();
    let pkcs7: PKCS7 = serde_json::from_str(&input).unwrap();
    dbg!(&pkcs7);
    // получаем данные необходимые для проверки
    let signature_algorithm_identifier = pkcs7.SignerInfos.SignatureAlgorithmIdentifier.clone();
    let signature_server = pkcs7
        .SignerInfos
        .UnsignedAttributes
        .SetOfAttributeValue
        .clone()
        .unwrap()
        .Signature;
    let server_open_key = pkcs7
        .SignerInfos
        .UnsignedAttributes
        .SetOfAttributeValue
        .clone()
        .unwrap()
        .Certificate;
    let message = pkcs7.EncapsulatedContentInfo.OCTET_STRING_OPTIONAL.clone();
    let digest_algorithm_identifier = pkcs7.DigestAlgorithmIdentifiers.clone();
    let user_signature = pkcs7.SignerInfos.SignatureValue.clone();
    let time = pkcs7
        .SignerInfos
        .UnsignedAttributes
        .SetOfAttributeValue
        .clone()
        .unwrap()
        .Timestamp
        .UTCTime;

    // собираем сообщение для которого будем считать хеш, чтобы проверить его (такое же формирование как на сервере)
    let new_message = message.clone() + user_signature.as_str();

    let chosen_hash_func: HashFunc;

    // считаем хеш сообщения в соответствии с id хеш функции
    let mut hash_server_message = match digest_algorithm_identifier.as_str() {
        "SHA256" => {
            println!("Функция хеширования SHA256");
            chosen_hash_func = HashFunc::Sha256;
            sha256(new_message.as_bytes())
                .iter()
                .map(|e| format!("{:02x}", e))
                .collect::<String>()
        },
        "SHA512" => {
            println!("Функция хеширования SHA512");
            chosen_hash_func = HashFunc::Sha512;
            sha512(new_message.as_bytes())
                .iter()
                .map(|e| format!("{:02x}", e))
                .collect::<String>()
        },
        "STREEBOG256" => {
            println!("Функция хеширования STREEBOG256");
            chosen_hash_func = HashFunc::Streebog256;
            streebog_256(new_message.as_bytes())
                .iter()
                .map(|e| format!("{:02x}", e))
                .collect::<String>()
        },
        "STREEBOG512" => {
            println!("Функция хеширования STREEBOG512");
            chosen_hash_func = HashFunc::Streebog512;
            streebog_512(new_message.as_bytes())
                .iter()
                .map(|e| format!("{:02x}", e))
                .collect::<String>()
        },
        _ => {
            chosen_hash_func = HashFunc::Streebog256;
            "".to_string()
        },
    };

    // добавляем к хешу метку времени utc_time, потому что на сервере мы подписываем хеш от функции + метку
    hash_server_message += pkcs7
        .SignerInfos
        .UnsignedAttributes
        .SetOfAttributeValue
        .clone()
        .unwrap()
        .Timestamp
        .UTCTime
        .as_str();

    // проверяем подпись в соответствии с id системы шифрования
    match signature_algorithm_identifier.as_str() {
        "RSA" => {
            println!("Функция шифрования RSA");
            // создаем систему rsa только на открытом ключе
            let rsa_for_validate = RSA::create(
                PrivateKey::from_raw_parts("0", "0", "0"),
                PublicKey::create_key_from_hashmap(server_open_key),
            );
            // расшифровываем подпись на открытом ключе шифрования
            let hash_for_verification =
                rsa_for_validate.decrypt_msg(TypeKey::PublicKey, &(signature_server));
            // проверяем совпадение
            if hash_for_verification == hash_server_message {
                println!("Подпись ЦШВ - корректна");
                // если корректность совпадает сохраняем данные в файл
                pkcs7.save_to_json("completed_pkcs7.json");
            }
        }

        "ElGamal" => {
            println!("Функция шифрования ElGamal");
            let (y, delta) = signature_server.split_once("|").unwrap();
            let mut el_gamal = ElGamal::default();
            let public_key = hand_made_el_gamal::PublicKey::from_hashmap(server_open_key);
            el_gamal.set_public_key(public_key);
            if el_gamal.check_signature((y.to_string(), delta.to_string()), &hash_server_message) {
                println!("Подпись ЦШВ для ElGamal корректна");
                pkcs7.save_to_json("completed_pkcs7.json");
            }
        }

        "FiatShamir" => {
            println!("Функция шифрования FiatShamir");
            let mut fiat_shamir = FiatShamir::default();
            let public_key = hand_made_fiat_shamir::PublicKey::from_hashmap(server_open_key);
            fiat_shamir.set_public_key(public_key);
            fiat_shamir.set_hash_func(chosen_hash_func);
            if fiat_shamir.check_signature(signature_server, new_message + time.as_str()) {
                println!("Подпись ЦШВ для FiatShamir корректна");
                pkcs7.save_to_json("completed_pkcs7.json");
            }
        }
        _ => (),
    };
}
