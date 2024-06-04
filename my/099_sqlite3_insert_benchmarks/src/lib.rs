use rand::prelude::SliceRandom;
use rand::Rng;
use smallstr::SmallString;

pub fn get_random_age() -> i8 {
    let vs: Vec<i8> = vec![5, 10, 15];
    *vs.choose(&mut rand::thread_rng()).unwrap()
}

pub fn get_random_active() -> i8 {
    if rand::random() {
        return 1;
    }
    0
}

pub fn get_random_bool() -> bool {
    rand::random()
}

pub fn get_random_area_code() -> SmallString<[u8; 6]> {
    use std::fmt::Write;
    let mut buf = SmallString::<[u8; 6]>::new();
    let mut rng = rand::thread_rng();
    write!(buf, "{:06}", rng.gen_range(0..999999)).unwrap();
    buf
}

pub fn get_random_area_code_large() -> SmallString<[u8; 256]> {
    use std::fmt::Write;
    let mut buf = SmallString::<[u8; 256]>::new();
    let mut rng = rand::thread_rng();
    write!(buf, "{:06}", rng.gen_range(0..999999)).unwrap();
    buf
}

pub fn pragma_rules() -> &'static str {
    // PRAGMA journal_mode = OFF;
    // PRAGMA synchronous = 0;
    // PRAGMA cache_size = 1000000;
    // PRAGMA locking_mode = EXCLUSIVE;
    // PRAGMA temp_store = MEMORY;
    // PRAGMA synchronous = normal;\
    "\
        PRAGMA journal_mode = WAL;\
        PRAGMA synchronous = normal;\
        PRAGMA foreign_keys = on;\
    "
}
