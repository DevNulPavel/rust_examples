use serde::Deserialize;
use crate::errors::MessageError;


pub async fn open_direct_message_channel(client: &reqwest::Client, api_token: &str, user_id: &str) -> Result<String, MessageError>{
    // Выполняем POST запрос
    let response = {
        let post_params = vec![
            ("user", user_id),
        ];

        client.post("https://slack.com/api/im.open")
            .bearer_auth(api_token)
            .form(&post_params)
            .send()
            .await?
    };
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