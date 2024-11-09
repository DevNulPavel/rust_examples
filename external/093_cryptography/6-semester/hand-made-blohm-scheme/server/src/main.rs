use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use nalgebra::{DMatrix};
use rand::{Rng, thread_rng};
use server::{MatrixWrapper, Data};

const P: u32 = 3571;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:9090").unwrap();
    let mut random_symmetric_matrix = Arc::new(generate_random_symmetric_matrix(10, P));
    let mut count_users = 0;

    //let mut buffer = [0u8;1024];

    let mut streams = Arc::new(Mutex::new(vec![]));

    while let Ok((stream, _)) = listener.accept(){
        count_users += 1;
        let stream = Arc::new(Mutex::new(stream));
        streams.lock().unwrap().push(stream.clone());
        let cloned_matrix = Arc::clone(&random_symmetric_matrix);
        println!("Выполнено подключение, создаём открытый и закрытый ключ для пользователя!");

        let cloned_streams = Arc::clone(&streams);

        std::thread::spawn(move || {
            handle_connection(Arc::clone(&stream), cloned_matrix);
        });

    }
}

fn handle_connection(mut tcp_stream: Arc<Mutex<TcpStream>>, symmetric_matrix: Arc<DMatrix<u32>>) {
    let open_key = generate_open_user_key(10, P);
    let close_key = &*symmetric_matrix * &open_key;
    let close_key = close_key.map(|el| el % P);

    let wrapper_open_key = MatrixWrapper::from(&open_key);
    let wrapper_close_key = MatrixWrapper::from(&close_key);
    let data = Data::new(P, wrapper_open_key, wrapper_close_key);

    tcp_stream.lock().unwrap().write_all(serde_json::to_string(&data).unwrap().as_bytes()).unwrap();
}

fn generate_open_user_key(k: usize, module: u32) -> DMatrix<u32> {
    let mut rng = thread_rng();
    let mut matrix = DMatrix::zeros(k,1);

    for i in 0..k {
        for j in 0..1 {
            let value = rng.gen_range(0..1000);
            matrix[(i, j)] = value % module;
        };
    };
    matrix
}

fn generate_random_symmetric_matrix(k: usize, module: u32) -> DMatrix<u32> {
    let mut rng = thread_rng();
    let mut matrix = DMatrix::zeros(k, k);

    for i in 0..k {
        for j in 0..=i {
            let value = rng.gen_range(0..1000);
            matrix[(i, j)] = value % module;
            matrix[(j, i)] = value % module;
        }
    }

    matrix
}