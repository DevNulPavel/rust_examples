use std::io::Read;
use std::net::TcpListener;
use client::Data;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9090").unwrap();
    let mut buffer = [0u8;1024];
    let mut last_hash= vec![];

    while let Ok((mut stream, _)) = listener.accept() {

        while let Ok(size) = stream.read(&mut buffer) {

            match size {
                0 => break,
                32 => {
                    last_hash = buffer[..32].to_vec();
                }
                _ => {
                    let data: Data = serde_json::from_str(&String::from_utf8_lossy(&buffer[..size]).to_string()).unwrap();
                    let index = data.get_index();
                    let hash= data.get_hash();
                    if hand_made_sha::sha256(&hash) == *last_hash {
                        println!("Successful");
                        println!("User index: {index}, last saved hash : {:?}, current hash: {:?}", &last_hash, hash);
                    }
                    last_hash = hash.to_vec();
                }
            }

        }

    }

}
