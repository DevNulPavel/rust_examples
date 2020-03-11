use serde::Deserialize;
use crate::errors::MessageError;

pub async fn find_user_id_by_email(client: &reqwest::Client, api_token: &str, email: &str) -> Result<String, MessageError> {
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