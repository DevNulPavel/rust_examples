extern crate bincode;
extern crate itertools;
extern crate rand;
extern crate rustc_serialize;

mod disk_btree;
mod multi_map;
mod wal_file;

use disk_btree::OnDiskBTree;
use multi_map::MultiMap;
use wal_file::{KeyValuePair, RecordFile};

use rustc_serialize::{Decodable, Encodable};

use itertools::merge;
use std::error::Error;

/// Максимальный размер элементов индекса в оперативной памяти
const MAX_MEMORY_ITEMS: usize = 1000;

// Трейты для поддерживаемых типов ключей и значений
pub trait KeyType: Ord + Encodable + Decodable + Clone {}
pub trait ValueType: Ord + Encodable + Decodable + Clone {}

/// Реализации автоматические для произвольных типов, реализующих данные трейты
impl<T> KeyType for T where T: Ord + Encodable + Decodable + Clone {}
impl<T> ValueType for T where T: Ord + Encodable + Decodable + Clone {}

/// This struct holds all the pieces of the BTree mechanism
pub struct BTree<K: KeyType, V: ValueType> {
    /// Путь к файлику дерева
    tree_file_path: String,
    /// Размер ключа в байтах
    key_size: usize,
    /// Размер значения в байтах
    value_size: usize,
    /// Write ahead файлик для лога
    wal_file: RecordFile<K, V>,
    /// Мапа в оперативной памяти, которая мержится с деревом на диске
    mem_tree: MultiMap<K, V>,
    /// Непосредственно сам файлик, который хранит все на диске
    tree_file: OnDiskBTree<K, V>,
}

impl<K: KeyType, V: ValueType> BTree<K, V> {
    pub fn new(
        tree_file_path: &String,
        key_size: usize,
        value_size: usize,
    ) -> Result<BTree<K, V>, Box<dyn Error>> {
        // Создаем мапу в оперативной памяти
        let mut mem_tree = MultiMap::<K, V>::new();

        // construct the path to the WAL file for the in-memory multi-map
        let wal_file_path = tree_file_path.to_owned() + ".wal";

        // construct our WAL file
        let mut wal_file = RecordFile::<K, V>::new(&wal_file_path, key_size, value_size)?;

        // if we have a WAL file, replay it into the mem_tree
        if wal_file.is_new()? {
            for kv in &mut wal_file {
                mem_tree.insert(kv.key, kv.value);
            }
        }

        // open the data file
        let tree_file = OnDiskBTree::<K, V>::new(tree_file_path.to_owned(), key_size, value_size)?;

        Ok(BTree {
            tree_file_path: tree_file_path.clone(),
            key_size,
            value_size,
            tree_file,
            wal_file,
            mem_tree,
        })
    }

    /// Inserts a key into the BTree
    pub fn insert(&mut self, key: K, value: V) -> Result<(), Box<Error>> {
        let record = KeyValuePair {
            key: key,
            value: value,
        };

        // should wrap this in a read-write lock
        try!(self.wal_file.insert_record(&record));

        let KeyValuePair { key, value } = record;

        let size = self.mem_tree.insert(key, value);

        if size > MAX_MEMORY_ITEMS {
            try!(self.compact());
        }

        return Ok(());
    }

    pub fn get(&self, key: &K) -> Option<std::collections::btree_set::Iter<V>> {
        self.mem_tree.get(key).map(|btree| btree)
    }

    /// Merges the records on disk with the records in memory
    fn compact(&mut self) -> Result<(), Box<Error>> {
        // create a new on-disk BTree
        let mut new_tree_file = try!(OnDiskBTree::<K, V>::new(
            self.tree_file_path.to_owned() + ".new",
            self.key_size,
            self.value_size
        ));

        // get an iterator for the in-memory items
        let mem_iter = self.mem_tree.into_iter();

        // get an iterator to the on-disk items
        let disk_iter = self.tree_file.into_iter();

        for kv in merge(mem_iter, disk_iter) {
            try!(new_tree_file.insert_record(&kv));
        }

        Ok(())
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use rand::{thread_rng, Rng};
    use std::collections::BTreeSet;
    use std::fs;
    use std::fs::OpenOptions;
    use BTree;

    pub fn gen_temp_name() -> String {
        let file_name: String = thread_rng().gen_ascii_chars().take(10).collect();

        return String::from("/tmp/") + &file_name + &String::from(".btr");
    }

    fn remove_files(file_path: String) {
        fs::remove_file(&file_path);
        fs::remove_file(file_path + ".wal");
    }

    #[test]
    fn new_blank_file() {
        let file_path = gen_temp_name();

        let btree = BTree::<u8, u8>::new(&file_path, 1, 1).unwrap();

        // make sure our two files were created
        let btf = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(&file_path)
            .unwrap();
        assert!(btf.metadata().unwrap().len() == 0);

        let wal = OpenOptions::new()
            .read(true)
            .write(false)
            .create(false)
            .open(file_path.to_owned() + ".wal")
            .unwrap();
        assert!(wal.metadata().unwrap().len() == 0);

        // make sure they think they're new too
        assert!(btree.wal_file.is_new().unwrap());
        assert!(btree.wal_file.count().unwrap() == 0);

        assert!(btree.tree_file.is_new().unwrap());
        assert!(btree.tree_file.count().unwrap() == 0);

        remove_files(file_path); // remove files assuming it all went well
    }

    #[test]
    fn new_existing_file() {
        let file_path = gen_temp_name();

        // scoped so it is cleaned up
        {
            BTree::<u8, u8>::new(&file_path, 1, 1).unwrap();
        }

        let btree = BTree::<u8, u8>::new(&file_path, 1, 1).unwrap();

        // check our file lengths from the struct
        assert!(btree.tree_file.count().unwrap() == 0);
        assert!(btree.wal_file.count().unwrap() == 0);

        remove_files(file_path); // remove files assuming it all went well
    }

    #[test]
    fn insert_new_u8() {
        let file_path = gen_temp_name();

        let mut btree = BTree::<u8, u8>::new(&file_path, 1, 1).unwrap();

        btree.insert(2, 3).unwrap(); // insert into a new file

        assert!(btree.wal_file.count().unwrap() == 1);
        assert!(btree.mem_tree.contains_key(&2));

        remove_files(file_path); // remove files assuming it all went well
    }

    #[test]
    fn insert_new_str() {
        let file_path = gen_temp_name();

        let mut btree = BTree::<String, String>::new(&file_path, 15, 15).unwrap();

        // insert into a new file
        btree
            .insert("Hello".to_owned(), "World".to_owned())
            .unwrap();

        assert!(!btree.wal_file.is_new().unwrap());
        assert!(btree.mem_tree.contains_key(&String::from("Hello")));

        remove_files(file_path); // remove files assuming it all went well
    }

    #[test]
    fn get_returns_an_iter() {
        let file_path = gen_temp_name();

        // setup tree
        let mut btree = BTree::<String, String>::new(&file_path, 15, 15).unwrap();

        // expected return set
        let mut expected: BTreeSet<String> = BTreeSet::new();
        expected.insert("World".to_string());

        btree.insert("Hello".to_owned(), "World".to_owned());

        // get the set at the hello key
        let set_at_hello: Vec<String> = btree.get(&"Hello".to_string()).unwrap().cloned().collect();

        assert_eq!(set_at_hello, ["World".to_string()]);

        remove_files(file_path); // remove files assuming it all went well
    }

    #[test]
    fn insert_multiple() {
        let file_path = gen_temp_name();

        let mut btree = BTree::<String, String>::new(&file_path, 15, 15).unwrap();

        // insert into a new file
        btree
            .insert("Hello".to_owned(), "World".to_owned())
            .unwrap();
        assert!(!btree.wal_file.is_new().unwrap());

        btree
            .insert("Hello".to_owned(), "Everyone".to_owned())
            .unwrap();
        assert!(!btree.wal_file.is_new().unwrap());

        remove_files(file_path); // remove files assuming it all went well
    }
}
