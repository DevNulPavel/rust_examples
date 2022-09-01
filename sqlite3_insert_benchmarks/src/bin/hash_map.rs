// id INTEGER not null primary key AUTOINCREMENT,
// area CHAR(6),
// age INTEGER not null,
// active INTEGER not null)",

use nohash_hasher::BuildNoHashHasher;
use smallstr::SmallString;
use sqlite3_insert_benchmarks as common;
use std::{collections::HashMap, mem::size_of};

// 16 bytes total
#[derive(Default)]
#[repr(align(64))]
#[allow(unused)]
struct Data {
    id: u64,
    area: SmallString<[u8; 14]>,
    age: i8,
    active: bool,
}

fn main() {
    println!("Size of struct: {}", size_of::<Data>());

    let count = 1_000_000;

    //let mut map: HashMap<u64, Data> = HashMap::with_capacity(count);
    let mut map: HashMap<u64, Data, _> =
        HashMap::with_capacity_and_hasher(count, BuildNoHashHasher::<u64>::default());

    for id in 0..1_000_000 {
        let area = common::get_random_area_code();
        let age = common::get_random_age();
        let active = common::get_random_active() == 0;

        let d = Data {
            active,
            age,
            area: area.into(),
            id,
        };

        map.insert(id, d);
    }
    assert_eq!(map.len(), count);
}
