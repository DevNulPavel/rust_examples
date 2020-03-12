use serde::Deserialize;
use serde::Serialize;


// Создаем структурки, в которых будут нужные значения
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub real_name: Option<String>
}

// Result<Vec<UserInfo>, Box<dyn std::error::Error>>
pub async fn iter_by_slack_users(client: &reqwest::Client, 
                                 api_token: &str, 
                                 last_cursor: Option<String>) -> (Option<String>, Vec<UserInfo>) {   
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
        //error: Option<String> // Раскомментить для парсинга ошибки
    }

    let mut result: Vec<UserInfo> = Vec::new();

    // Выполняем GET запрос 
    let request_res = {
        // Создаем курсор с ссылкой
        let last_cursor_ptr = last_cursor
            .as_ref()
            .map(|val|{
                val.as_str()
            });
        // Создаем список параметров
        let get_parameters: [(&str, Option<&str>); 3] = [
            ("token", Some(api_token)),
            ("limit", Some("150")),
            ("cursor", last_cursor_ptr)
        ];
        //println!("{:?}", get_parameters);            
        client.get("https://slack.com/api/users.list")
            .query(&get_parameters)
            .send()
            .await
    };

    // Проверка запроса
    let parse_res = match request_res {
        Ok(request_res) => {
            // Парсинг
            request_res.json::<UsersResponse>().await
        }
        Err(_) => return (None, result),
    };
    // Проверка парсинга
    let json: UsersResponse = match parse_res {
        Ok(json) => {
            // Json валидный
            json
        },
        Err(_) => return (None, result),
    };

    // Сам Json распарсился нормально
    if !json.ok {
        return (None, result);
    }

    // Обрабатываем пользователей
    match json.members {
        Some(mut members) => {
            result.append(members.as_mut());
        },
        None => return (None, result),
    }

    // Метадата
    let meta: Metadata = match json.response_metadata {
        Some(meta) => {
            // Валидная метадата
            meta
        },
        None => return (None, result),
    };

    // Следующий курсор
    match meta.next_cursor {
        Some(cur) => {
            return (Some(cur), result);
        },
        None => return (None, result),
    };
}