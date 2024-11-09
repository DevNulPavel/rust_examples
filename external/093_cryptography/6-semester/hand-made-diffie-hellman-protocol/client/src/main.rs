use std::io::{Read, Write};
use std::net::TcpStream;
use num_bigint::{BigInt, Sign};
use num_primes::Generator;
use rand::thread_rng;
use num_bigint::RandBigInt;

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:9090").unwrap();
    let p = gen_prime(512);
    stream.write_all(&p.to_bytes_be().1).unwrap();

    let w = "secret key";
    let hash_w = hand_made_sha::sha256(w.as_bytes());

    let mut alpha = thread_rng().gen_bigint_range(&BigInt::from(1), &BigInt::from(&p-1));

    while alpha.modpow(&BigInt::from(2), &p) == BigInt::from(1) || alpha.modpow(&(&p/2), &p) == BigInt::from(1) || alpha.modpow(&BigInt::from(2), &p) == BigInt::from(&p-1){
        alpha = thread_rng().gen_bigint_range(&BigInt::from(1), &BigInt::from(&p-1));
    }

    //создаем g из секретного ключа w который знают только 2 пользователя
    let alpha = BigInt::from_bytes_be(Sign::Plus, &hash_w);

    stream.write_all(&alpha.to_bytes_be().1).unwrap();

    let x = thread_rng().gen_bigint_range(&BigInt::from(2), &(&p-2));

    let a = alpha.modpow(&x,&p);
    stream.write_all(&a.to_bytes_be().1).unwrap();

    let mut buffer = [0u8;1024];

    let size = stream.read(&mut buffer).unwrap();
    let b = BigInt::from_bytes_be(Sign::Plus, &buffer[..size]);

    let k = b.modpow(&x, &p);
    println!("k: {k}");
}

fn gen_prime(bits: usize) -> BigInt {
    Generator::new_prime(bits)
        .to_str_radix(10)
        .parse::<BigInt>()
        .unwrap()
}