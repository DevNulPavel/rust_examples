use rand::{distributions::Alphanumeric, Rng};
pub use std::collections::{HashMap, HashSet};

type Key = [u8; 16];
type Value = [u8; 16];

/// The amount of dummy data bytes in `BigStruct`.
const DUMMY_DATA_COUNT: usize = 512;

/// The type of the dummy data field in `BigStruct`.
/// If you set dummy data to a static size array: `type DummyData = [u8; DUMMY_DATA_COUNT];`
/// there is no difference between iter and non-iter fills, not sure why.
type DummyData = Vec<u8>;

/// A struct to hold the `Value` for `Key` plus some dummy data to bloat the memory.
pub struct BigStruct {
    pub uuid: Value,
    pub dum: DummyData,
}

/// The type of the collection to collect the `(Key, BigStruct)` pairs into in `fill_map_iter` & `fill_map`.
/// Try to set it to `Vec<(Key, BigStruct)>`, yields different results.
type CollectionType = HashMap<Key, BigStruct>;
// type CollectionType = Vec<(Key, BigStruct)>;

/// Создаем итератор рандомных `(Key, BigStruct)` пар.
pub fn generate_random_pairs(amount: u32) -> impl Iterator<Item = (Key, BigStruct)> {
    // Генератор рандома
    let mut rng = rand::thread_rng();

    // Создаем вектор из нужных рандомных значений размером DUMMY_DATA_COUNT = 512
    let dum: DummyData = (&mut rng)
        .sample_iter(Alphanumeric)
        .take(DUMMY_DATA_COUNT)
        .collect::<Vec<u8>>();

    (0..amount).map(move |_| {
        let key = uuid::Uuid::new_v4().into_bytes();

        let val = BigStruct {
            uuid: uuid::Uuid::new_v4().into_bytes(), // This is the value we will use for inverse map.
            dum: dum.clone(),                        // Extra dummy data to bloat the memory.
        };

        (key, val)
    })
}

/// Заполняется только обратная хешмапа без предварительного какого-то сохранения заранее.
/// Чисто собираем значения из итератора.
pub fn fill_map_light(inverse_map: &mut HashMap<Value, HashSet<Key>>, amount: u32) {
    // Итератор по рандомным значениям
    let iter = generate_random_pairs(amount);

    for (key, val_s) in iter {
        let (key, val) = (key, val_s.uuid);

        // Получаем из обратной мапы значение хешсета если уже есть
        if let Some(set) = inverse_map.get_mut(&val) {
            // Добавляем туда ключ
            set.insert(key);
        } else {
            // Либо добавляем новый сет с первым элементом
            inverse_map.insert(val, HashSet::from_iter(std::iter::once(key)));
        }
    }
}

/// Fills the inverse map by collecting the random data iterator then iterating over it
/// using `iter()`.
/// This takes the most memory but is trimmable to the same size as `fill_map_light`.
pub fn fill_map_iter(inverse_map: &mut HashMap<Value, HashSet<Key>>, amount: u32) {
    let map: CollectionType = generate_random_pairs(amount).collect();

    for (key, val_s) in map.iter() {
        let (key, val) = (*key, val_s.uuid);
        if let Some(set) = inverse_map.get_mut(&val) {
            set.insert(key);
        } else {
            inverse_map.insert(val, HashSet::from_iter(vec![key]));
        }
    }
}

/// Fills the inverse map by collecting the random data iterator then consuming it
/// and filling it's content directly in the inverse map.
/// This takes some memory slightly less than `fill_map_iter`, but the downside is that it's
/// not trimmable, i.e. It uses about 30% more memory compared to `fill_map_light` and this memory
/// isn't freed until the inverse map is freed, which can be a big problem if the inverse map is long-lived.
pub fn fill_map(inverse_map: &mut HashMap<Value, HashSet<Key>>, amount: u32) {
    let map: CollectionType = generate_random_pairs(amount).collect();
    // let mut bin = vec![];

    for (key, val_s) in map {
        let (key, val) = (key, val_s.uuid);
        if let Some(set) = inverse_map.get_mut(&val) {
            set.insert(key);
        } else {
            inverse_map.insert(val, HashSet::from_iter(vec![key]));
        }

        // bin.push(val_s);
    }
}