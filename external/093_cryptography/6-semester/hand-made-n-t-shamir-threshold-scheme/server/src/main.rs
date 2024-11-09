use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex, mpsc};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use serde::{Serialize, Deserialize};
use std::sync::Arc;
use std::collections::HashMap;
use polynomial::Polynomial;
use rand::{Rng, thread_rng};
use server::{gen_random_polynomial, lagrange_interpolation};

const P: u32 = 3571;
const T: usize = 3;

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    Register,
    RequestKey {user_id: usize, key: u32},
    ResponseKey {server_data: u32, users_id: Vec<usize>},
    RequestData{from: usize},
    ResponseData { from: usize, to: usize, data: u32},
    Error{message: String},
    // id пользователя который создал дату, и сама дата
}
#[derive(Serialize, Deserialize, Debug)]
struct UserData {
    pub user_id: usize,
    pub p: u32,
    pub k: u32,
}

type ClientList = Arc<Mutex<HashMap<usize, mpsc::Sender<Message>>>>;

async fn handle_client(mut socket: TcpStream, clients: ClientList, user_id: usize, poly: Polynomial<i32>, secret_parts: Arc<Mutex<Vec<(u32, u32)>>>) {
    let (mut reader, mut writer) = socket.split();
    let mut buffer = vec![0; 1024];
    let mut users_response = vec![];

    println!("Пользователь зарегистрирован с id {user_id}");
    let user_data = UserData {
        user_id,
        p: P,
        k: poly.eval(user_id as i32) as u32,
    };
    println!("Часть секрета пользователя: {}, id: {user_id}", user_data.k);
    let (tx, mut rx) = mpsc::channel(32);

    loop {
        tokio::select! {
            n = reader.read(&mut buffer) => {
                let n = n.unwrap_or(0);
                if n == 0 {
                    break;
                }
                let msg: Message = serde_json::from_slice(&buffer[..n]).unwrap();
                match msg {
                    Message::Register => {
                        clients.lock().await.insert(user_data.user_id, tx.clone());
                        writer.write_all(&serde_json::to_vec(&user_data).unwrap()).await.unwrap();
                    },
                    Message::ResponseData {from, to, data} => {
                        if secret_parts.lock().await.len() == T {
                            let xs = secret_parts.lock().await.iter().map(|(x,_)|*x).collect::<Vec<u32>>();
                            let ys = secret_parts.lock().await.iter().map(|(_,y)|*y).collect::<Vec<u32>>();
                            let secret_key = lagrange_interpolation(&xs, &ys).data()[0].round() as u32;
                            clients.lock().await[&to].send(Message::ResponseKey {users_id: users_response.clone(), server_data: secret_key}).await.unwrap();
                            *secret_parts.lock().await = vec![];
                        }
                        else {
                            secret_parts.lock().await.push((from as u32, data));
                        }
                    }
                    Message::RequestKey {user_id, key} => {
                        println!("Запрошено восстанавление секрета!");
                        secret_parts.lock().await.push((user_id as u32, key));
                        for (user, tx) in clients.lock().await.iter() {
                            if *user != user_data.user_id {
                                tx.send(Message::RequestData {from: user_id}).await.unwrap();
                            }
                        }
                    }
                    _ => {}
                }
            },
            Some(msg) = rx.recv() => {
                let serialized = serde_json::to_vec(&msg).unwrap();
                writer.write_all(&serialized).await.unwrap();
            }
        }
    }


    //clients.lock().await.remove(&user_id);
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    let clients: ClientList = Arc::new(Mutex::new(HashMap::new()));
    let mut next_id = thread_rng().gen_range(0..P);
    let mut random_poly = gen_random_polynomial(T-1);
    println!("Секрет: {}", random_poly.data()[0]);
    let mut secret_parts = Arc::new(Mutex::new(vec![]));

    loop {
        let secret_parts_clone = Arc::clone(&secret_parts);
        let poly_clone = random_poly.clone();
        let id = next_id;
        next_id = thread_rng().gen_range(0..P/2);

        let (socket, _) = listener.accept().await.unwrap();
        let clients = clients.clone();
        tokio::spawn(async move {
            handle_client(socket, clients, id.try_into().unwrap(), poly_clone, secret_parts_clone).await;
        });
    }
}
