use parking_lot::RwLock;
use std::{hint::black_box, time::Duration};

////////////////////////////////////////////////////////////////////////////////

type TestData = [u8; 144];

////////////////////////////////////////////////////////////////////////////////

const COUNT: usize = 60_000_000;

////////////////////////////////////////////////////////////////////////////////

fn main() {
    {
        let mut slab = slab::Slab::with_capacity(COUNT);

        {
            let write_begin = std::time::Instant::now();

            for _ in 0..COUNT {
                slab.insert([0_u8; 144]);
            }

            let write_elapsed = std::time::Instant::now().duration_since(write_begin);

            println!("Write duration: {}msec", write_elapsed.as_millis());
        }

        {
            let read_begin = std::time::Instant::now();

            for _val in slab.iter() {
                black_box(_val);
                assert!(*_val.1.get(5).unwrap() == 0);
            }

            let read_elapsed = std::time::Instant::now().duration_since(read_begin);

            println!("Read duration: {}msec", read_elapsed.as_millis());
        }

        println!("Sleep");
        std::thread::sleep(Duration::from_secs(5));
    }

    {
        let mut slab = slab::Slab::with_capacity(COUNT);

        {
            let write_begin = std::time::Instant::now();

            for _ in 0..COUNT {
                slab.insert(RwLock::new([0_u8; 144]));
            }

            let write_elapsed = std::time::Instant::now().duration_since(write_begin);

            println!("Write duration: {}msec", write_elapsed.as_millis());
        }

        {
            let read_begin = std::time::Instant::now();

            std::thread::scope(|s| {
                s.spawn(|| {
                    for val in slab.iter() {
                        let _lock = val.1.read();

                        let _lock_v = black_box(_lock);

                        assert!(*_lock_v.get(5).unwrap() == 0);
                    }
                });

                s.spawn(|| {
                    for val in slab.iter() {
                        let _lock = val.1.read();

                        let _lock_v = black_box(_lock);

                        assert!(*_lock_v.get(15).unwrap() == 0);
                    }
                });

                s.spawn(|| {
                    for val in slab.iter() {
                        let _lock = val.1.read();

                        let _lock_v = black_box(_lock);

                        assert!(*_lock_v.get(45).unwrap() == 0);
                    }
                });

                s.spawn(|| {
                    for val in slab.iter() {
                        let _lock = val.1.read();

                        let _lock_v = black_box(_lock);

                        assert!(*_lock_v.get(8).unwrap() == 0);
                    }
                });
            });

            let read_elapsed = std::time::Instant::now().duration_since(read_begin);

            println!("Read duration: {}msec", read_elapsed.as_millis());
        }

        println!("Sleep");
        std::thread::sleep(Duration::from_secs(5));
    }

    {
        let mut slab = sharded_slab::Slab::new();

        {
            let write_begin = std::time::Instant::now();

            for _ in 0..COUNT {
                slab.insert(RwLock::new([0_u8; 144]));
            }

            let write_elapsed = std::time::Instant::now().duration_since(write_begin);

            println!("Write duration: {}msec", write_elapsed.as_millis());
        }

        {
            let read_begin = std::time::Instant::now();

            for val in slab.unique_iter() {
                let _lock = val.read();

                let _lock_v = black_box(_lock);

                assert!(*_lock_v.get(5).unwrap() == 0);
            }

            let read_elapsed = std::time::Instant::now().duration_since(read_begin);

            println!("Read duration: {}msec", read_elapsed.as_millis());
        }

        println!("Sleep");
        std::thread::sleep(Duration::from_secs(5));
    }
}
