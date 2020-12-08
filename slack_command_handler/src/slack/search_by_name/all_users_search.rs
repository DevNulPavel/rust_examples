use serde::Deserialize;
use serde::Serialize;
use crate::{
    slack::{
        SlackRequestBuilder
    }
};


// Создаем структурки, в которых будут нужные значения
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UserInfo {
    pub id: String,
    pub name: String,
    pub real_name: Option<String>
}

impl PartialEq<UserInfo> for UserInfo {
    fn eq(&self, other: &UserInfo) -> bool {
        self.id == other.id &&
        self.name == other.name &&
        self.real_name == other.real_name
    }
}

// Result<Vec<UserInfo>, Box<dyn std::error::Error>>
pub async fn iter_by_slack_users(client: &SlackRequestBuilder, 
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

        /*#[derive(Deserialize, Serialize, Debug)]
        enum Parameter<'a>{
            Str(&'a str),
            Numb(i32),
            Cursor(Option<&'a str>)
        }
        // Создаем список параметров
        let get_parameters: [(&str, Parameter<'_>); 3] = [
            ("token", Parameter::Str(api_token)),
            ("limit", Parameter::Str("150")),
            ("cursor", Parameter::Cursor(last_cursor_ptr))
        ];
        println!("Test: {:?}", get_parameters);*/

        /*let get_parameters = serde_json::json!([
            ["token", api_token],
            ["limit", "150"],
            ["cursor", last_cursor_ptr]
        ]);
        println!("{:?}", serde_json::to_string(&get_parameters));*/

        // Создаем список параметров
        let get_parameters: [(&str, Option<&str>); 2] = [
            //("token", Some(api_token)),
            ("limit", Some("150")),
            ("cursor", last_cursor_ptr)
        ];

        client
            .build_get_request("https://slack.com/api/users.list")
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
        Ok(json) if json.ok => {
            // Json валидный и флаг там ok
            json
        },
        Ok(_) => return (None, result),
        Err(_) => return (None, result),
    };

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