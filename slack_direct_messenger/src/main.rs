#![allow(dead_code)]

extern crate tokio;
extern crate hyper;
extern crate reqwest;
extern crate clap;

use std::cmp::Ordering;
use std::collections::HashMap;
use clap::{Arg, App};
use tokio::prelude::*;
use tokio::net::TcpListener;
use hyper::Client;
use serde::Deserialize; // Serialize


async fn test_tokio_server() -> Result<(), Box<dyn std::error::Error>>{
    // Создаем серверный сокет
    let mut listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        // В цикле получаем новые подключения
        let (mut socket, _) = listener.accept().await?;

        // Закидываем на обработку новые асинхронные функции для обработки подключения
        tokio::spawn(async move {
            // Создаем буффер
            let mut buf = [0; 1024];

            // Начинаем читать в цикле из сокета
            loop {
                // Пробуем прочитать в сокет    
                let n = match socket.read(&mut buf).await {
                    // Сокет у нас закрыт, прочитали 0 данных - значит выходим из данного обработчика сокета
                    Ok(n) if (n == 0) => {
                        return;
                    },
                    // Если у нас ненулевое значение прочитано, значит все ок
                    Ok(n) => {
                        n
                    },
                    // Если ошибка, выводим ее и выходим из обработчика сокета
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                };

                // Пишем данные назад, в случае ошибки - выводим ее и выходим
                if let Err(e) = socket.write_all(&buf[0..n]).await {
                    eprintln!("failed to write to socket; err = {:?}", e);
                    return;
                }
            }
        });
    }
    //Ok(())
}

async fn perform_get_request_with_hyper() -> Result<(), Box<dyn std::error::Error>> {
    // Still inside `async fn main`...
    let client = Client::new();

    // Parse an `http::Uri`...
    let uri = "http://httpbin.org/ip".parse()?;

    // Await the response...
    let resp = client.get(uri).await?;

    println!("Response status: {}", resp.status());

    // And now...
    // while let Some(chunk) = resp.body_mut().data().await {
    //     stdout().write_all(&chunk?).await?;
    // }

    Ok(())
}

async fn perform_get_request_with_reqwest() -> Result<(), Box<dyn std::error::Error>> {
    // let resp: HashMap<String, String> = reqwest::get("https://httpbin.org/ip")
    //     .await?
    //     .json()
    //     .await?;
    // println!("{:#?}", resp);

    // let body = reqwest::get("https://www.rust-lang.org")
    //     .await?
    //     .text()
    //     .await?;
    //println!("body = {:?}", body);

    //https://docs.rs/reqwest/0.10.0-alpha.2/reqwest/

    // This will POST a body of `{"lang":"rust","body":"json"}`
    let mut map = HashMap::new();
    map.insert("lang", "rust");
    map.insert("body", "json");
    let client = reqwest::Client::new();
    let res = client.post("http://httpbin.org/post")
        .json(&map)
        .send()
        .await?;
    println!("result = {:?}", res);

    Ok(())
}

async fn find_user_id_by_email(api_token: &str, email: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Проверяем наличие email
    if email.is_empty(){
        return Err(Box::from("Is empty email"));
    }

    // Выполняем GET запрос
    let get_parameters = vec![
        ("token", api_token), 
        ("email", email)
    ];
    let client = reqwest::Client::new();
    let response = client.get("https://slack.com/api/users.lookupByEmail")
        .query(&get_parameters)
        .send()
        .await?;
    //println!("{:?}", response);

    // Создаем структурки, в которых будут нужные значения
    #[derive(Deserialize, Debug)]
    struct UserInfo {
        id: String,
    }
    #[derive(Deserialize, Debug)]
    struct UserResponse {
        ok: bool,
        user: UserInfo,
    }

    // Парсим ответ в json
    let response_json = response
        .json::<UserResponse>()
        .await?;
    //println!("{:?}", response_json);
    
    // Результат, если все ок
    if response_json.ok {
        return Ok(response_json.user.id);
    }

    return Err(Box::from(format!("User is not found for this email: {}", email)));
}

//////////////////////////////////////////////////////////////////////////////////////////////////

// Создаем структурки, в которых будут нужные значения
#[derive(Deserialize, Debug, Clone)]
struct UserInfo {
    id: String,
    name: String,
    real_name: Option<String>
}

// Result<Vec<UserInfo>, Box<dyn std::error::Error>>
async fn request_all_slack_users(api_token: &str) -> Vec<UserInfo> {   
    // Создаем структурки, в которых будут нужные значения
    #[derive(Deserialize, Debug)]
    struct Metadata {
        next_cursor: Option<String>
    }
    #[derive(Deserialize, Debug)]
    struct UsersResponse {
        ok: bool,
        response_metadata: Option<Metadata>,
        members: Option<Vec<UserInfo>>,
        error: Option<String>
    }

    let mut result: Vec<UserInfo> = Vec::new();

    let mut last_cursor: Option<String> = None;
    loop {
        let mut get_parameters: Vec<(&str, String)> = vec![
            ("token", String::from(api_token)),
            ("limit", "150".to_owned()) // Можно создавать владеющую строку вот так
        ];
        if let Some(last_cursor_val) = last_cursor.take() { // Take, нужен чтобы забрать значение из Option и сделать его None
            get_parameters.push(("cursor", last_cursor_val));
        }

        //println!("{:?}", get_parameters);

        // Выполняем GET запрос
        let client = reqwest::Client::new();
        let request_res = client.get("https://slack.com/api/users.list")
            .query(&get_parameters)
            .send()
            .await;

        //println!("{:?}", request_res);
        
        // Если сервер ответил
        if let Ok(response) = request_res {
            // Пробуем распарсить результат
            let mut parse_res = response
                .json::<UsersResponse>()
                .await;

            //println!("{:?}", parse_res);

            if let Ok(json) = parse_res.as_mut() {
                //println!("{:?}", json);
                if json.ok == true {
                    //println!("{:?}", json);

                    // Обрабатываем пользователей
                    if let Some(users) = json.members.take().as_mut() {
                        result.append(users);
                    }else{
                        // Завершаем цикл при ошибке запроса
                        break;
                    }

                    // Обновляем курсор, если он есть
                    last_cursor = if let Some(meta) = json.response_metadata.take().as_mut() {
                        if let Some(cursor) = meta.next_cursor.take() {
                            if cursor.is_empty() == false {
                                Some(cursor)
                            }else{
                                break;
                            }
                        }else{
                            // Завершаем цикл при ошибке запроса
                            break;
                        }
                    }else{
                        // Завершаем цикл при ошибке запроса
                        break;
                    };
                }else{
                    // Завершаем цикл при ошибке запроса
                    break;
                }
            }else{
                // Завершаем цикл при ошибке запроса
                break;
            }
        }else{
            // Завершаем цикл при ошибке запроса
            break;
        }
    }

    result
}

// TODO: Поправить возвращаемое значение
async fn find_user_id_by_name(api_token: &str, src_user_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Проверяем наличие user
    if src_user_name.is_empty(){
        return Err(Box::from("Is empty user"));
    }

    let user = src_user_name.to_lowercase();

//     user = user.toLowerCase();
//     if (!USERS_CACHE) {
//         const fullPath = path.join(CACHE_FILE_FOLDER, CACHE_FILE_NAME);
//         const exists = fs.existsSync(fullPath);
//         if (exists) {
//             try{
//                 const rawdata = fs.readFileSync(fullPath);
//                 USERS_CACHE = JSON.parse(rawdata.toString());    
//             }catch(_){
//                 USERS_CACHE = {};
//             }
//         }else {
//             USERS_CACHE = {};
//         }
//     }
//     const foundUserInfo = USERS_CACHE[user];
//     if (foundUserInfo) {
//         return foundUserInfo.id;
//     }

    let mut found_info: Option<UserInfo> = None;

    let users_list = request_all_slack_users(api_token).await;
    if users_list.len() == 0 {
        return Err(Box::from("Empty users list"));
    }

    struct UserInfoWithPriority{
        priority: i32,
        info: UserInfo,
    }
    let mut found_users: Vec<UserInfoWithPriority> = Vec::new(); 

    let search_parts: Vec<&str> = user.split(" ").collect();
    for user_info in users_list { // Объекты будут перемещать владение в user_info
        // Проверяем короткое имя
        if user_info.name == user {
            found_info = Some(user_info);
            break;
        }else{
            // Проверяем полное имя
            if let Some(ref real_name_src) = user_info.real_name {
                let real_name = real_name_src.to_lowercase();

                // Нашли сразу же
                if real_name == user {
                    found_info = Some(user_info);
                    break;
                }else{
                    let mut possible_val = UserInfoWithPriority{
                        priority: 0,
                        info: user_info,
                    };
                    let test_name_components = real_name.split(" "); // split создает итератор с &str
                    for test_part in test_name_components { // Здесь у нас владение перемещается полностью в итерируемые элементы, test_name_components пустой после
                        for search_part in search_parts.iter() { // Тут мы итерируемся по элементам
                            if test_part == *search_part {
                                possible_val.priority += 1;
                            }
                        }
                    }
    
                    if possible_val.priority > 0 {
                        found_users.push(possible_val);
                    }
                }
            }
        }
    }
    
    if let Some(info) = found_info{
        return Ok(info.id);
    }else{
        found_users.sort_by(|val1, val2|-> Ordering {
            if val1.priority > val2.priority{
                return Ordering::Greater;
            } else if val1.priority < val2.priority{
                return Ordering::Less;
            }
            return Ordering::Equal;
        });

        if let Some(user) = found_users.first(){
            return Ok(user.info.id.clone());
        }
    }


//     if (foundInfo) {
//         USERS_CACHE[user] = foundInfo;
//     }else if (foundUsers.length > 0) {
//         const sortedUsers = foundUsers.sort((a, b) => {
//             if (a.priority && b.priority) {
//                 return b.priority - a.priority;
//             }
//             if (a.priority) {
//                 return 0 - a.priority;
//             }
//             if (b.priority) {
//                 return b.priority - 0;
//             }
//             return 999999999;
//         });
//         foundInfo = sortedUsers[0];
//         delete foundInfo.priority;
//         USERS_CACHE[user] = foundInfo;
//     }

//     if (foundInfo) {
//         if(fs.existsSync(CACHE_FILE_FOLDER) === false){
//             fs.mkdirSync(CACHE_FILE_FOLDER, { recursive: true });
//         }

//         const fullPath = path.join(CACHE_FILE_FOLDER, CACHE_FILE_NAME);
//         const data = JSON.stringify(USERS_CACHE, null, 4);
//         fs.writeFileSync(fullPath, data);
        
//         return foundInfo.id;
//     }

    return Err(Box::from("User id is not found"));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // let result = test_tokio_server();
    // result.await?;

    //perform_get_request_with_hyper().await?;
    
    //perform_get_request_with_reqwest().await?;

    // Parse parameters
    let matches = App::new("slack_direct_messenger")
                            .version("1.0")
                            .author("Pavel Ershov")
                            .about("Send direct messages to slack")
                            .arg(Arg::with_name("email")
                                .long("slack_user_email")
                                .help("Slack user email"))
                            .arg(Arg::with_name("user")
                                .long("slack_user")
                                .help("Username"))
                            .arg(Arg::with_name("text")
                                .long("slack_user_text")
                                .help("Text"))
                            .arg(Arg::with_name("qr_comment")
                                .long("slack_user_qr_commentary")
                                .help("QR code commentary"))
                            .arg(Arg::with_name("qr_text")
                                .long("slack_user_qr_text")
                                .help("QR code text"))
                          .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let email = matches.value_of("email").unwrap_or("");
    let user = matches.value_of("user").unwrap_or("");
    //let text = matches.value_of("text").unwrap_or("");
    //let qr_commentary = matches.value_of("qr_comment").unwrap_or("");
    //let qr_text = matches.value_of("qr_text").unwrap_or("");

    // Api token
    let api_token = std::env::var("SLACK_API_TOKEN").expect("SLACK_API_TOKEN environment variable is missing");

    let id = match find_user_id_by_email(api_token.as_str(), email).await {
        Ok(id) => id,
        Err(_)=>{
            match find_user_id_by_name(api_token.as_str(), user).await {
                Ok(id) => id,
                Err(err) => {
                    println!("{}", err);
                    return Err(Box::from("Failed to get user id"));
                }
            }
        }
    };
    println!("{}", id);

    Ok(())
}