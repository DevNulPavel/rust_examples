use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;
use super::super::super::errors::MessageError; // Можно даже так вместо crate::
use super::all_users_search::{
    UserInfo,
    iter_by_slack_users
};

fn read_cache(cache_file_full_path: &Path) -> HashMap<String, UserInfo> {
    let users_cache: HashMap<String, UserInfo> = File::open(cache_file_full_path)
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
    
    users_cache
}

fn save_cache(cache_file_folder: &Path, cache_file_full_path: &Path, users_cache: HashMap<String, UserInfo> ) {
    if !cache_file_folder.exists() {
        std::fs::create_dir_all(cache_file_folder).unwrap();
    }
    if let Ok(mut file) = File::open(cache_file_full_path) {
        if let Ok(json_text) = serde_json::to_string(&users_cache){
            if file.write_all(json_text.as_bytes()).is_ok(){
                println!("Write success");
            }
        }
    }
}

fn search_by_fullname(full_users_list: Vec<UserInfo>, user: &str) -> Option<UserInfo>{
    // Структура юзера с приоритетом
    #[derive(Debug)]
    struct UserInfoWithPriority{
        priority: i32,
        info: UserInfo,
    }

    let mut found_users: Vec<UserInfoWithPriority> = Vec::new(); 

    let search_parts: Vec<&str> = user.split(' ').collect();
    for user_info in full_users_list { // Объекты будут перемещать владение в user_info        
        // Проверяем полное имя
        if let Some(ref real_name_src) = user_info.real_name {
            let real_name = real_name_src.to_lowercase();

            // Нашли сразу же
            if real_name == user {
                return Some(user_info);
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

    // TODO: работает неправильно!
    /*found_users.sort_by_key(|user: &UserInfoWithPriority|{
        -user.priority
    });*/

    found_users.sort_by(|val1, val2|-> std::cmp::Ordering {
        if val1.priority < val2.priority{
            return std::cmp::Ordering::Greater;
        } else if val1.priority > val2.priority{
            return std::cmp::Ordering::Less;
        }
        std::cmp::Ordering::Equal
    });

    // Вернем просто первый элемент
    return found_users
        .into_iter()
        .take(1)
        .next()
        .map(|user_info| user_info.info);
    /*for user_info in found_users {
        return Some(user_info.info);
    }*/
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
    let mut users_cache: HashMap<String, UserInfo> = read_cache(&cache_file_full_path);

    // Ищем в кеше
    /*let entry = users_cache.entry(user);
    if let std::collections::hash_map::Entry::Occupied(entry) = entry{
        return Ok(entry.remove().id)
    }*/
    if let Some(found) = users_cache.get(&user){
        return Ok(found.id.clone()); // TODO: Избавиться от клонирования, можно мувить результат
    }

    // Создаем новый объект результата
    let found_info: Option<UserInfo> = {
        let mut found_info_local: Option<UserInfo> = None;
        let mut full_users_list: Vec<UserInfo> = Vec::new();

        let mut last_cursor = Option::None;
        loop{
            // Получаем список юзеров итерационно
            let (new_cursor, mut users_list) = iter_by_slack_users(client, api_token, last_cursor).await;

            // Нет юзеров - конец
            if users_list.is_empty() {
                break;
            }

            // Проверяем короткое имя
            found_info_local = users_list
                .iter()
                .find(|user_info|{
                    user_info.name == user
                })
                .map(|val| {
                    val.clone()
                });

            // Нашли - все ок
            if found_info_local.is_some() {
                break;
            }

            // Если не нашлось - сохраняем для полного поиска
            full_users_list.append(&mut users_list);

            // Сохраняем курсор для новой итерации
            last_cursor = new_cursor;

            // Если нет нового курсора - заканчиваем итерации
            if last_cursor.is_none(){
                break;
            }
        }

        // Если поиск по короткому имени не отработал, пробуем по полному имени
        if found_info_local.is_none(){
            found_info_local = search_by_fullname(full_users_list, &user);
        }
        
        found_info_local
    };
    
    if let Some(info) = found_info{
        // Добавляем найденного пользователя в кэш
        users_cache.insert(user.to_owned(), info.clone());

        //println!("{:?}", users_cache);
        // Сохраняем наш кэш
        // TODO: !!!
        save_cache(&cache_file_folder, &cache_file_full_path, users_cache);

        println!("{:?}", info);
        return Ok(String::new());
        //return Ok(info.id);
    }

    Err(MessageError::IsNotFound)
}