#![warn(clippy::all)]
#![allow(dead_code)]

#[macro_use(defer)] 
extern crate scopeguard;
extern crate tokio;
extern crate hyper;
extern crate reqwest;
extern crate clap;
extern crate dirs;
extern crate serde_json;
extern crate qrcode;
extern crate image;
extern crate uuid;

use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
//use std::cmp::Ordering;
use std::collections::HashMap;
use clap::{Arg, App};
use serde::Deserialize;
use serde::Serialize;
use qrcode::QrCode;
use image::Luma;

// Тип используемых ошибок в коде
#[derive(Debug)]
enum MessageError{
    EmptyEmail,
    EmptyUser,
    IsNotFound,
    UsersListIsMissing,
    ChannelDidNotOpen,
    RequestError(reqwest::Error) // Обертка над ошибкой запроса
}

// Пустая автоматическая реализация
impl std::error::Error for MessageError{
}

// Код для автоматической конвертации ошибки в коде
impl std::fmt::Display for MessageError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Код для автоматической конвертации ошибки в коде
impl From<reqwest::Error> for MessageError {
    fn from(err: reqwest::Error) -> Self {
        Self::RequestError(err)
    }
}


// Создаем структурки, в которых будут нужные значения
#[derive(Deserialize, Serialize, Debug, Clone)]
struct UserInfo {
    id: String,
    name: String,
    real_name: Option<String>
}

async fn find_user_id_by_email(client: &reqwest::Client, api_token: &str, email: &str) -> Result<String, MessageError> {
    // Проверяем наличие email
    if email.is_empty(){
        return Err(MessageError::EmptyEmail);
    }

    // Выполняем GET запрос
    let get_parameters = vec![
        ("token", api_token), 
        ("email", email)
    ];
    let response = client
        .get("https://slack.com/api/users.lookupByEmail")
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

    Err(MessageError::IsNotFound)
}

// Result<Vec<UserInfo>, Box<dyn std::error::Error>>
async fn request_all_slack_users(client: &reqwest::Client, api_token: &str) -> Vec<UserInfo> {   
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
                if json.ok {
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
                            if !cursor.is_empty() {
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
async fn find_user_id_by_name(client: &reqwest::Client, api_token: &str, src_user_name: &str) -> Result<String, MessageError> {
    // Проверяем наличие user
    if src_user_name.is_empty(){
        return Err(MessageError::EmptyUser);
    }

    // Переводим имя в нижний регистр
    let user = src_user_name.to_lowercase();
        
    // Пути к папке с кешем пользователей
    let cache_file_folder = PathBuf::new()
        .join(dirs::home_dir().unwrap())
        .join(".cache/slack_direct_messenger/");
    let cache_file_name = Path::new("users_cache.json");    
    let cache_file_full_path = PathBuf::new()
        .join(&cache_file_folder)
        .join(&cache_file_name);

    // Читаем данные из файлика, если не удалось - просто создаем новый
    let mut users_cache: HashMap<String, UserInfo> = File::open(&cache_file_full_path)
        .and_then(|mut file: std::fs::File|{
            let mut data = String::new();
            file.read_to_string(&mut data)?;
            Ok(data)
        })
        .and_then(|data: String|{
            let result = serde_json::from_str::<HashMap<String, UserInfo>>(&data)?;
            Ok(result)
        })
        .unwrap_or_default();

    // Ищем в кеше
    if let Some(found) = users_cache.get(&user){
        return Ok(found.id.clone()); // TODO: Избавиться от клонирования, можно мувить результат
    }

    // Создаем новый объект результата
    let mut found_info: Option<UserInfo> = None;

    // Получаем список юзеров
    let users_list = request_all_slack_users(client, api_token).await;
    if users_list.is_empty() {
        return Err(MessageError::UsersListIsMissing);
    }

    // Структура юзера с приоритетом
    struct UserInfoWithPriority{
        priority: i32,
        info: UserInfo,
    }

    let mut found_users: Vec<UserInfoWithPriority> = Vec::new(); 

    let search_parts: Vec<&str> = user.split(' ').collect();
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
                    let test_name_components = real_name.split(' '); // split создает итератор с &str
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
    
    let mut result_found_user: Option<UserInfo> = None;
    if let Some(info) = found_info{
        users_cache.insert(user.to_owned(), info.clone());
        result_found_user = Some(info);
    }else{
        found_users.sort_by_key(|user: &UserInfoWithPriority|{
            user.priority
        });
        /*found_users.sort_by(|val1, val2|-> Ordering {
            if val1.priority > val2.priority{
                return Ordering::Greater;
            } else if val1.priority < val2.priority{
                return Ordering::Less;
            }
            return Ordering::Equal;
        });*/

        for user_info in found_users {
            users_cache.insert(user.clone(), user_info.info.clone()); // TODO: !!!
            result_found_user = Some(user_info.info);
        }
    }

    // Сохраняем наш кэш
    if let Some(user_info) = result_found_user{
        if !cache_file_folder.exists() {
            std::fs::create_dir_all(cache_file_folder).unwrap();
        }
        if let Ok(mut file) = File::open(&cache_file_full_path) {
            if let Ok(json_text) = serde_json::to_string(&users_cache){
                if file.write_all(json_text.as_bytes()).is_ok(){
                }
            }
        }

        return Ok(user_info.id);
    }

    Err(MessageError::IsNotFound)
}

async fn open_direct_message_channel(client: &reqwest::Client, api_token: &str, user_id: &str) -> Result<String, MessageError>{
    // Выполняем POST запрос
    let post_params = vec![
        ("user", user_id),
    ];
    let response = client.post("https://slack.com/api/im.open")
        .bearer_auth(api_token)
        .form(&post_params)
        .send()
        .await?;
    //println!("{:?}", response);
    
    // Создаем структурки, в которых будут нужные значения
    #[derive(Deserialize, Debug)]
    struct ChannelInfo {
        id: String,
    }
    #[derive(Deserialize, Debug)]
    struct ResponseInfo {
        ok: bool,
        channel: ChannelInfo,
    }

    // Парсим ответ в json
    let response_json = response
        .json::<ResponseInfo>()
        .await?;
    //println!("{:?}", response_json);
    
    // Результат, если все ок
    if response_json.ok {
        return Ok(response_json.channel.id);
    }

    Err(MessageError::ChannelDidNotOpen)
}

async fn send_direct_message_to_channel(client: &reqwest::Client, api_token: &str, channel: &str, text: &str) -> Result<(), MessageError>{
    // Выполняем POST запрос
    let post_params = vec![
        ("channel", channel),
        ("text", text)
    ];
    client.post("https://slack.com/api/chat.postMessage")
        .bearer_auth(api_token)
        .form(&post_params)
        .send()
        .await?;
    //println!("{:?}", response);
    
    Ok(())
}

async fn send_qr_to_channel(client: &reqwest::Client, api_token: &str, channel: &str, qr_text: &str, qr_commentary: &str) -> Result<(), MessageError>{
    // Encode some data into bits.
    let code = QrCode::new(qr_text.as_bytes()).unwrap();

    // Render the bits into an image.
    let image = code.render::<Luma<u8>>().build();

    // File path
    let new_uuid = uuid::Uuid::new_v4();
    let filename = format!("{}.png", new_uuid);
    let temp_dir = std::env::temp_dir();
    let result_file_path = PathBuf::new()
        .join(temp_dir)
        .join(filename.as_str());
    //println!("{:?}", result_file_path);

    // Save the image.
    image.save(&result_file_path).unwrap();

    // Сразу прописываем отложенное удаление
    defer!(std::fs::remove_file(&result_file_path).unwrap());

    // TODO: Читаем файлик
    let file_data: Vec<u8> = File::open(&result_file_path)
        .and_then(|mut file|{
            let mut data: Vec<u8> = Vec::new();
            file.read_to_end(&mut data)?;
            Ok(data)
        })
        .unwrap(); // Если не прочитали файлик - падаем

    //println!("{:?}", file_data);

    // Есть или нет комментарий?
    let commentary = match qr_commentary.len() {
        0 => qr_commentary,
        _ => qr_text
    };

    use reqwest::multipart::Part;
    use reqwest::multipart::Form;

    let post_params = vec![
        ("channel", channel.to_owned()),
        ("initial_comment", commentary.to_owned()),
        ("filename", filename.to_owned()),
    ];
    let form = Form::new()
        .part("channel", Part::text(channel.to_owned()))
        .part("initial_comment", Part::text(commentary.to_owned()))
        .part("filename", Part::text(filename.to_owned()))
        .part("file", Part::bytes(file_data))
        // .part("content", Part::stream(file_data))
        .percent_encode_attr_chars();
    //println!("{}", form.boundary());

    let _response = client.post("https://slack.com/api/files.upload")
        .bearer_auth(api_token)
        .form(&post_params)
        .multipart(form)
        .send()
        .await?;
    //println!("{:?}", _response);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse parameters
    let matches = App::new("slack_direct_messenger")
                            .version("1.0")
                            .author("Pavel Ershov")
                            .about("Send direct messages to slack")
                            .arg(Arg::with_name("email")
                                .long("slack_user_email")
                                .help("Slack user email")
                                .takes_value(true))
                            .arg(Arg::with_name("user")
                                .long("slack_user")
                                .help("Username")
                                .takes_value(true))
                            .arg(Arg::with_name("text")
                                .long("slack_user_text")
                                .help("Text")
                                .takes_value(true))
                            .arg(Arg::with_name("qr_comment")
                                .long("slack_user_qr_commentary")
                                .help("QR code commentary")
                                .takes_value(true))
                            .arg(Arg::with_name("qr_text")
                                .long("slack_user_qr_text")
                                .help("QR code text")
                                .takes_value(true))
                            .get_matches();

    // Gets a value for config if supplied by user, or defaults to "default.conf"
    let email = matches.value_of("email").unwrap_or("");
    let user = matches.value_of("user").unwrap_or("");
    let text = matches.value_of("text").unwrap_or("");
    let qr_commentary = matches.value_of("qr_comment").unwrap_or("");
    let qr_text = matches.value_of("qr_text").unwrap_or("");

    // Api token
    let api_token = std::env::var("SLACK_API_TOKEN").expect("SLACK_API_TOKEN environment variable is missing");

    // Создаем клиента для переиспользования
    let client = reqwest::Client::new();

    // Ищем id
    let id = match find_user_id_by_email(&client, &api_token, email).await {
        Ok(id) => id,
        Err(_)=>{
            match find_user_id_by_name(&client, &api_token, user).await {
                Ok(id) => id,
                Err(err) => {
                    println!("{}", err);
                    return Err(Box::from("Failed to get user id"));
                }
            }
        }
    };
    //println!("{}", id);

    // Открываем канал для сообщений
    let channel_id = open_direct_message_channel(&client, &api_token, &id).await?;
    //println!("{}", channel_id);
    
    if !text.is_empty() {
        // String можно преобразовать в &String, затем вызовется as_ref() -> &str
        send_direct_message_to_channel(&client, &api_token, &channel_id, text).await?;
    }

    if !qr_text.is_empty() {
        send_qr_to_channel(&client, &api_token, &channel_id, qr_text, qr_commentary).await?;
    }

    Ok(())
}