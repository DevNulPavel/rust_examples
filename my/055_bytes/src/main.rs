use std::{
    time::{
        Duration
    },
    thread::{
        sleep
    }
};
use bytes::{
    Bytes,
    BytesMut
};


fn test_vec(){
    let mut data = Vec::new();
    data.resize(1024 * 1024 * 4, 0);

    let initial_buffer = Bytes::from(data);

    let mut buffers = Vec::new();
    loop {
        let new_buffer = initial_buffer.clone();
        buffers.push(new_buffer);

        println!("New allocation");

        sleep(Duration::from_secs(2));  
    }
}

fn test_mut_buf(){
    let mut data = BytesMut::new();
    data.resize(1024 * 1024 * 4, 0);

    let initial_buffer = data.freeze();

    let mut buffers = Vec::new();
    loop {
        let new_buffer = initial_buffer.clone();
        buffers.push(new_buffer);

        println!("New allocation");

        sleep(Duration::from_secs(2));  
    }
}

fn test_mut_buf_2(){
    let mut data = BytesMut::new();
    data.resize(1024 * 1024 * 4, 0);

    let initial_buffer = data.freeze();

    let mut buffers = Vec::new();
    loop {
        let new_buffer = Bytes::from(initial_buffer.clone());
        buffers.push(new_buffer);

        println!("New allocation");

        sleep(Duration::from_secs(2));  
    }
}

fn main() {
    // test_mut_buf();
    test_mut_buf_2();
}