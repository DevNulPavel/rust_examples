use std::io::{Read, Write};
use std::net::TcpListener;
use num_bigint::{BigInt, RandBigInt, Sign};
use rand::thread_rng;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9090").unwrap();
    let mut buffer = [0u8;1024];
    let w = "secret key";
    let hash_w = hand_made_sha::sha256(w.as_bytes());

    while let Ok((mut stream, _)) = listener.accept(){

        let size = stream.read(&mut buffer).unwrap();

        let p = BigInt::from_bytes_be(Sign::Plus, &buffer[..size]);
        let size = stream.read(&mut buffer).unwrap();
        //создаем g из секретного ключа w который знают только 2 пользователя
        let alpha = BigInt::from_bytes_be(Sign::Plus, &hash_w);

        let y = thread_rng().gen_bigint_range(&BigInt::from(2), &(&p-2));

        let b = alpha.modpow(&y, &p);

        let size = stream.read(&mut buffer).unwrap();
        let a = BigInt::from_bytes_be(Sign::Plus, &buffer[..size]);

        stream.write_all(&b.to_bytes_be().1).unwrap();

        let k = a.modpow(&y, &p);

        println!("k: {k}");

    }

}
