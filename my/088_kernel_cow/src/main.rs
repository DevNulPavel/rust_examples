use std::{collections::VecDeque, sync::Arc, time::Duration};

fn main() {
    ////////////////////////////////////////////////////////////

    std::thread::sleep(Duration::from_secs(5));

    println!("Before fill");

    // 64 мегабайта суммарно
    const SIZE: usize = 1024 * 1024 * 64;

    let mut buffer = {
        let mut buffer_data = VecDeque::<u8>::with_capacity(SIZE);

        for i in 0..SIZE {
            buffer_data.push_back((i % 255) as u8);
        }

        Arc::new(buffer_data)
    };

    println!("After fill");

    ////////////////////////////////////////////////////////////

    std::thread::sleep(Duration::from_secs(5));

    println!("Before change");

    {
        assert_eq!(Arc::strong_count(&buffer), 1);
        let mutable_buffer = Arc::make_mut(&mut buffer);

        for i in 0..SIZE {
            (*mutable_buffer.get_mut(i).unwrap()) = (i % 255) as u8;
        }
    }

    println!("After change");

    ////////////////////////////////////////////////////////////

    std::thread::sleep(Duration::from_secs(5));

    println!("Before clone");

    let buffer_clone = buffer.clone();

    println!("After clone");

    ////////////////////////////////////////////////////////////

    std::thread::sleep(Duration::from_secs(5));

    println!("Before read");

    for i in 0..SIZE {
        assert_eq!(Arc::strong_count(&buffer_clone), 2);
        buffer_clone.get(i).unwrap();
    }

    println!("After read");

    ////////////////////////////////////////////////////////////

    std::thread::sleep(Duration::from_secs(5));

    println!("Before clone change");

    {
        assert_eq!(Arc::strong_count(&buffer), 2);
        
        // Здесь у нас происходит клонирование буффера, так как счетчик ссылок текущего буфера равен двум
        let mutable_buffer = Arc::make_mut(&mut buffer);

        for i in 0..SIZE {
            (*mutable_buffer.get_mut(i).unwrap()) = (i % 255) as u8;
        }
    }

    println!("After clone change");

    ////////////////////////////////////////////////////////////

    std::thread::sleep(Duration::from_secs(60));
}
