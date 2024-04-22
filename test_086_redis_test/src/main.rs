use redis::{Client, Commands};

macro_rules! format_small {
    ($len:expr, $($arg:tt)*) => {
        {
            use std::fmt::Write;
            let mut buf = smallstr::SmallString::<[u8; $len]>::new();
            buf.write_fmt(::core::format_args!($($arg)*)).unwrap();
            buf
        }
    };
}

fn main() {
    // redis-server --unixsocket /Users/devnul/redis.sock --unixsocketperm 775

    //let client = Client::open("redis://127.0.0.1:6379").unwrap();
    let client = Client::open("redis+unix:///Users/devnul/redis.sock").unwrap();

    let mut conn = client.get_connection().unwrap();

    /*for i in 0..4_000_000 {
        let key = format_small!(32, "u:{:010}", i);
        conn.set::<_, _, ()>(key.as_str(), i).unwrap();

        if i % 100_000 == 0 {
            println!("{}: {}", i, key.as_str());
        }
    }*/

    for i in 0..4_000_000 {
        let key2 = format_small!(32, "{:010}", i);
        conn.hset::<_, _, _, ()>("users", key2.as_str(), i).unwrap();

        if i % 100_000 == 0 {
            println!("{}", i);
        }
    }
}
