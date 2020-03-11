use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;
use serde::Deserialize;
use serde::Serialize;
use crate::errors::MessageError;

// Создаем структурки, в которых будут нужные значения
#[derive(Deserialize, Serialize, Debug, Clone)]
struct UserInfo {
    id: String,
    name: String,
    real_name: Option<String>
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

pub async fn find_user_id_by_name(client: &reqwest::Client, api_token: &str, src_user_name: &str) -> Result<String, MessageError> {
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