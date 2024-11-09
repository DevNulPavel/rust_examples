use std::io::{Read, Write};
use std::net::TcpStream;
use num_bigint::{BigInt, RandBigInt};
use num_primes::Generator;
use num_traits::ToBytes;
use rand::{thread_rng};

fn main() {
    let mut stream = TcpStream::connect("127.0.0.1:9090").unwrap();

    let p  = gen_prime(512);
    let q = gen_prime(512);
    let n = p * q;

    stream.write_all(&n.to_be_bytes()).unwrap();

    let mut s = thread_rng().gen_bigint_range(&BigInt::from(1), &(&n-1));
    while hand_made_math::expanded_euclidean_algorithm(s.clone(), n.clone()).0 != BigInt::from(1) {
        s = thread_rng().gen_bigint_range(&BigInt::from(1), &(&n-1));
    }

    let v = s.modpow(&BigInt::from(2), &n);

    stream.write_all(&v.to_be_bytes()).unwrap();

   for _ in 0..10 {
       let z = thread_rng().gen_bigint_range(&BigInt::from(1), &(&n-1));

       let x = z.modpow(&BigInt::from(2), &n);
       stream.write_all(&x.to_be_bytes()).unwrap();

       let mut buffer_c = [0u8; 1];
       if let Err(_) = stream.read(&mut buffer_c) {
           println!("Конец итераций, аутентификация пройдена успешно!");
           break;
       }
       let c = u8::from_be_bytes(buffer_c);

       if c == 0{
           stream.write_all(&z.to_be_bytes()).unwrap();
       }
       else if c == 1 {
           stream.write_all(&(z * &s % &n).to_be_bytes()).unwrap();
       }
   }

}

pub fn gen_prime(bits: usize) -> BigInt {
    Generator::new_prime(bits)
        .to_str_radix(10)
        .parse::<BigInt>()
        .unwrap()
}