use std::io::Write;
use std::net::TcpStream;
use hand_made_rsa::{RSA, TypeKey};
use rand::random;

fn main() {
    let session_key = "Hello_world!";
    let random_number: u32 = random();
    let rsa = RSA::generate_keys();
    let public_key = serde_json::to_string(&rsa.get_public_key()).unwrap();
    let private_key = serde_json::to_string(&rsa.get_private_key()).unwrap();
    let keys = public_key + "|" +  private_key.as_str();

    let mut stream = TcpStream::connect("127.0.0.1:9090").unwrap();

    stream.write_all(keys.as_bytes()).unwrap();

    let sign_data = vec![random_number.to_string(), session_key.to_string(), "B".to_string()].join("|");
    let sign = rsa.encrypt_msg(TypeKey::PrivateKey, &sign_data);

    let data_to_encrypt = vec![session_key.to_string(), random_number.to_string(), sign].join("|");
    let encrypted_message = rsa.encrypt_msg(TypeKey::PublicKey, &data_to_encrypt);

    stream.write_all(encrypted_message.as_bytes()).unwrap();

}
