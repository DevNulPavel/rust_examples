use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;
use std::path::Path;
use std::collections::HashMap;
use super::all_users_search::{
    UserInfo,
    iter_by_slack_users
};
use crate::{
    slack::{
        SlackRequestBuilder
    }
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
        .map_err(|e|{
            eprintln!("Try to read cache: {:?}", e);
            e
        })
        .unwrap_or_default();
    
    users_cache
}

fn save_cache(cache_file_folder: &Path, cache_file_full_path: &Path, users_cache: &HashMap<String, UserInfo> ) {
    if !cache_file_folder.exists() {
        std::fs::create_dir_all(cache_file_folder).unwrap();
    }
    //println!("{:?}", cache_file_full_path);

    #[derive(Debug)]
    enum SaveError{
        CantCreateFile(std::io::Error),
        CantConvertToJson(serde_json::Error),
        CantWriteFile(std::io::Error),
    }
    
    // Создаем файлик
    let error = File::create(cache_file_full_path)
        .map_err(|e|{
            // Конвертируем формат ошибки
            SaveError::CantCreateFile(e)
        })
        .and_then(|file|{
            // Сохраняем в строку
            serde_json::to_string(&users_cache)
                .map_err(|e|{
                    // Конвертируем формат ошибки
                    SaveError::CantConvertToJson(e)
                })
                .map(move |json_result| {
                    // Новый результат будет сотоять из файла и json
                    (file, json_result)
                })
        })
        .and_then(|(mut file, json_text)|{
            // Пишем в файлик
            file.write_all(json_text.as_bytes())
                .map_err(|e|{
                    // Конвертируем формат ошибки
                    SaveError::CantWriteFile(e)
                })
        })
        .err();

    if let Some(err) = error{
        eprintln!("Try to write cache: {:?}", err);
    }else{
        //println!("Write success");
    }
}

fn search_by_fullname(full_users_list: Vec<UserInfo>, user_lowercase: &str) -> Option<UserInfo>{
    // Структура юзера с приоритетом
    #[derive(Debug)]
    struct UserInfoWithPriority{
        priority: i32,
        info: UserInfo,
    }

    let mut found_users: Vec<UserInfoWithPriority> = Vec::new(); 

    let search_parts: Vec<&str> = user_lowercase.split(' ').collect();
    for user_info in full_users_list { // Объекты будут перемещать владение в user_info        
        // Проверяем полное имя
        if let Some(ref real_name_src) = user_info.real_name {
            let real_name = real_name_src.to_lowercase();

            // Нашли сразу же
            if real_name == user_lowercase {
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

pub async fn find_user_id_by_name(client: &SlackRequestBuilder, src_user_name: &str) -> Option<String> {
    // Проверяем наличие user
    if src_user_name.is_empty(){
        return None;
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
        return Some(found.id.clone()); // TODO: Избавиться от клонирования, можно мувить результат
    }

    // Создаем новый объект результата
    let found_info: Option<UserInfo> = {
        let mut full_users_list: Vec<UserInfo> = Vec::new();

        let mut last_cursor = Option::None;
        // У цикла можно указать метку, затем с помощью break можно прервать работу именно этого цикла
        let mut found_info_local: Option<UserInfo> = 'tag: loop{
            // Получаем список юзеров итерационно
            let (new_cursor, mut users_list) = iter_by_slack_users(&client, last_cursor).await;

            // Нет юзеров - конец
            if users_list.is_empty() {
                break 'tag None;
            }

            // Проверяем короткое имя
            let found_info_local = users_list
                .iter()
                .find(|user_info|{
                    user_info.name == user
                })
                .map(|val| {
                    val.clone()
                });

            // Нашли - все ок
            if found_info_local.is_some() {
                break 'tag found_info_local;
            }

            // Если не нашлось - сохраняем для полного поиска
            full_users_list.append(&mut users_list);

            // Сохраняем курсор для новой итерации
            last_cursor = new_cursor;

            // Если нет нового курсора - заканчиваем итерации
            if last_cursor.is_none(){
                break 'tag None;
            }
        };

        // Если поиск по короткому имени не отработал, пробуем по полному имени
        if found_info_local.is_none(){
            found_info_local = search_by_fullname(full_users_list, &user);
        }
        
        found_info_local
    };
    
    if let Some(info) = found_info{
        // Добавляем найденного пользователя в кэш
        users_cache.insert(user.to_owned(), info.clone());

        // Сохраняем наш кэш
        save_cache(&cache_file_folder, &cache_file_full_path, &users_cache);

        //println!("{:?}", info);
        return Some(info.id);
    }

    None
}

// этот модуль включается только при тестировании mod tests {
#[cfg(test)]
mod tests{
    use super::*;

    fn generate_test_users() -> HashMap<String, UserInfo>{
        let mut users_cache: HashMap<String, UserInfo> = HashMap::new();
        users_cache.insert(String::from("pershov"), UserInfo{
            id: String::from("asdasd"),
            name: String::from("pershov"),
            real_name: Some(String::from("Pavel Ershov"))
        });
        users_cache.insert(String::from("Pavel Ershov"), UserInfo{
            id: String::from("asdasd"),
            name: String::from("pershov"),
            real_name: Some(String::from("Pavel Ershov"))
        });
        users_cache.insert(String::from("Ershov Pavel"), UserInfo{
            id: String::from("asdasd"),
            name: String::from("pershov"),
            real_name: Some(String::from("Pavel Ershov"))
        });
        users_cache.insert(String::from("Pavel Ivanov"), UserInfo{
            id: String::from("ggggg"),
            name: String::from("pivanov"),
            real_name: Some(String::from("Pavel Ivanov"))
        });
        users_cache.insert(String::from("pivanov"), UserInfo{
            id: String::from("ggggg"),
            name: String::from("pivanov"),
            real_name: Some(String::from("Pavel Ivanov"))
        });
        users_cache.insert(String::from("Ivanov Pavel"), UserInfo{
            id: String::from("ggggg"),
            name: String::from("pivanov"),
            real_name: Some(String::from("Pavel Ivanov"))
        });
        users_cache.insert(String::from("Test Name"), UserInfo{
            id: String::from("gfdg"),
            name: String::from("tname"),
            real_name: Some(String::from("Test Name"))
        });
        users_cache.insert(String::from("Name Test"), UserInfo{
            id: String::from("fgdfg"),
            name: String::from("tname"),
            real_name: Some(String::from("Test Name"))
        });
        users_cache.insert(String::from("cake"), UserInfo{
            id: String::from("gdfgdfg"),
            name: String::from("cake"),
            real_name: None
        });
        
        users_cache
    }

    #[test]
    fn test_cache(){
        // Пути к папке с кешем пользователей
        let cache_file_folder = Path::new("./test_cache/");
        let cache_file_name = Path::new("users_cache.json");

        // Полный путь к файлику
        let cache_file_full_path = PathBuf::new()
            .join(cache_file_folder)
            .join(cache_file_name);
        
        // Создаем новый тестовый список
        let users_cache: HashMap<String, UserInfo> = generate_test_users();

        // Сохраняем в файликъ
        save_cache(&cache_file_folder, &cache_file_full_path, &users_cache);

        // Читаем из файлика
        let saved_users = read_cache(&cache_file_full_path);
        
        // Удаляем файлик
        let file_removed = std::fs::remove_dir_all(cache_file_folder).is_ok();
        assert!(file_removed);

        // Должны быть одинаковые
        assert_eq!(users_cache, saved_users);
    }

    #[test]
    fn test_full_name_serach(){
        let test_names_map = generate_test_users();
        let test_names_vec: Vec<UserInfo> = test_names_map
            .iter()
            .map(|(_, val)|{
                val.clone()
            })
            .collect();

        assert_eq!(search_by_fullname(test_names_vec.clone(), "pavel ershov").map(|val| val.id), 
                   Some(test_names_map["pershov"].id.clone()));
                
        assert_eq!(search_by_fullname(test_names_vec.clone(), "ershov pavel").map(|val| val.id), 
                   Some(test_names_map["pershov"].id.clone()));

        assert_eq!(search_by_fullname(test_names_vec.clone(), "user unknown").map(|val| val.id), 
                   None);

        assert_eq!(search_by_fullname(test_names_vec.clone(), "unknown").map(|val| val.id), 
                   None);
    }
}