// id INTEGER not null primary key AUTOINCREMENT,
// area CHAR(6),
// age INTEGER not null,
// active INTEGER not null)",

use std::mem::size_of;

use smallstr::SmallString;
use smallvec::Array;
use sqlite3_insert_benchmarks as common;

// const L: usize
fn ser_smal<S, A: Array<Item = u8>>(v: &SmallString<A>, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(v.as_str())
}

// 16 bytes total
#[repr(align(64))]
#[derive(serde::Serialize, Default)]
struct Data {
    id: u64,
    #[serde(serialize_with = "ser_smal")]
    area: SmallString<[u8; 6]>,
    age: i8,
    active: bool,
}

fn main() {
    println!("Size of struct: {}", size_of::<Data>());
    println!(
        "Size of serialized: {}",
        bincode::serialize(&Data::default()).unwrap().len()
    );

    let db = sled::open("sled_db_batch").unwrap();
    for _ in 0..1_000 {
        let mut batch = sled::Batch::default();
        for _ in 0..1_000 {
            let id = db.generate_id().unwrap();

            let area = common::get_random_area_code();
            let age = common::get_random_age();
            let active = common::get_random_active() == 0;

            let d = Data {
                active,
                age,
                area: area.into(),
                id,
            };

            let key: [u8; 8] = id.to_be_bytes();
            batch.insert(&key, bincode::serialize(&d).unwrap());
        }
        db.apply_batch(batch).unwrap();
    }
    db.flush().unwrap();
}
