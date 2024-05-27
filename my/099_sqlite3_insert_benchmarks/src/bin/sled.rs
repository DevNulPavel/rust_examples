// id INTEGER not null primary key AUTOINCREMENT,
// area CHAR(6),
// age INTEGER not null,
// active INTEGER not null)",

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

#[derive(serde::Serialize)]
struct Data {
    id: u64,
    #[serde(serialize_with = "ser_smal")]
    area: SmallString<[u8; 6]>,
    age: i8,
    active: bool,
}

fn main() {
    let db = sled::Config::default()
        .use_compression(false)
        .mode(sled::Mode::HighThroughput)
        // .use_compression(true)
        // .print_profile_on_drop(true)
        .path("sled_db")
        .open()
        .unwrap();
    // let db = sled::open("sled_db").unwrap();
    for _ in 0..1_000_000 {
        let id = db.generate_id().unwrap();
        let area = common::get_random_area_code();
        let age = common::get_random_age();
        let active = common::get_random_active() == 0;

        let d = Data {
            active,
            age,
            area,
            id,
        };

        db.insert(id.to_be_bytes(), bincode::serialize(&d).unwrap())
            .unwrap();
    }
    db.flush().unwrap();
}
