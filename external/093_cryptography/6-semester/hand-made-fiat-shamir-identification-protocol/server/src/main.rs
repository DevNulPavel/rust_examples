use std::io::{Read, Write};
use std::net::TcpListener;
use num_bigint::{BigInt, Sign};
use rand::Rng;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9090").unwrap();
    let mut buffer = [0u8;2048];
    let mut v = BigInt::from(0);
    let mut n = BigInt::from(0);

    while let Ok((mut stream, _)) = listener.accept() {
        let size = stream.read(&mut buffer).unwrap();
        n = BigInt::from_bytes_be(Sign::Plus, &buffer[..size]);
        println!("n: {n}");

        let size = stream.read(&mut buffer).unwrap();
        v = BigInt::from_bytes_be(Sign::Plus, &buffer[..size]);
        println!("v: {v}");

        while let Ok(size) = stream.read(&mut buffer) {

            if size == 0 {
                break
            }

            let x = BigInt::from_bytes_be(Sign::Plus, &buffer[..size]);
            println!("x: {x}");
            let c:i8 = rand::thread_rng().gen_range(0..=1);
            println!("c: {c}");
            stream.write_all(&c.to_be_bytes()).unwrap();

            let size = stream.read(&mut buffer).unwrap();
            let y = BigInt::from_bytes_be(Sign::Plus, &buffer[..size]);
            println!("y: {y}");

            if y != BigInt::from(0) && y.modpow(&BigInt::from(2), &n) == x * v.modpow(&BigInt::from(c), &n) {
                println!("Идентификация пройдена успешно!");
                break;
            }

        }


    }
}
