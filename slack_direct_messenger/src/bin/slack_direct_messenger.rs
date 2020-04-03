#![warn(clippy::all)]
#![allow(clippy::let_and_return)]
#![allow(dead_code)]


// Можно создавать отдельно библиотеку и отдельно код бинарника в папке bin
// В зависимости от имени файлика - создается исполняемый бинарник

use clap::{
    Arg, 
    App,
    ArgMatches
};
// Так мы подключаем библиотеку для использования
use slack_direct_messenger::{
    messaging,
    user_search
};

async fn find_user_id(client: &reqwest::Client, api_token: &str, user_email: &str, user_name: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Ищем id сначала по email, если не вышло - по имени

    // Сначала ищем пользователя по email
    let id = match user_search::find_user_id_by_email(&client, &api_token, user_email).await {
        Ok(id) => id,
        Err(_)=>{
            // Если не нашли - ищем по имени
            match user_search::find_user_id_by_name(&client, &api_token, user_name).await {
                Ok(id) => id,
                Err(err) => {
                    eprintln!("{}", err);
                    return Err(Box::from("Failed to get user id"));
                }
            }
        }
    };
    //println!("{}", id);
    Ok(id)
}

fn get_app_parameters() -> ArgMatches<'static> {
    // Parse parameters
    App::new("slack_direct_messenger")
        .version("1.0")
        .author("Pavel Ershov")
        .about("Send direct message to slack")
        // Channel
        .arg(Arg::with_name("slack_channel")
            .long("slack_channel")
            .help("Slack channel")
            .takes_value(true))
        // User
        .arg(Arg::with_name("slack_user_email")
            .long("slack_user_email")
            .help("Slack user email")
            .takes_value(true))
        .arg(Arg::with_name("slack_user")
            .long("slack_user")
            .help("Username")
            .takes_value(true))
        // Data
        .arg(Arg::with_name("text")
            .long("slack_user_text")
            .long("slack_channel_text")
            .help("Text")
            .takes_value(true))
        .arg(Arg::with_name("qr_commentary")
            .long("slack_user_qr_commentary")
            .long("slack_channel_qr_commentary")
            .help("QR code commentary")
            .takes_value(true))
        .arg(Arg::with_name("qr_text")
            .long("slack_user_qr_text")
            .long("slack_channel_qr_text")
            .help("QR code text")
            .takes_value(true))
        .get_matches()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = get_app_parameters();

    // Получаем значения фактические
    let channel = matches.value_of("slack_channel").unwrap_or("");
    let user_email = matches.value_of("slack_user_email").unwrap_or("");
    let user_name = matches.value_of("slack_user").unwrap_or("");
    let text = matches.value_of("text").unwrap_or("");
    let qr_commentary = matches.value_of("qr_commentary").unwrap_or("");
    let qr_text = matches.value_of("qr_text").unwrap_or("");

    // Api token
    let api_token = std::env::var("SLACK_API_TOKEN")
        .expect("SLACK_API_TOKEN environment variable is missing");

    // Создаем клиента для переиспользования
    let client = reqwest::Client::new();

    // Определяем в какой канал пишем
    let channel_id = if !channel.is_empty() {
        // Искать канал не надо, сразу отдаем канал и текст
        let channel = channel
            .find('#')
            .map(|pos|{
                // Берем часть строки после #
                channel
                    .split_at(pos)
                    .1
            })
            .unwrap_or(channel); // Если нету # - берем просто канал
        channel.to_string()
    }else if !user_email.is_empty() || !user_name.is_empty() {
        // Находм ID пользователя
        let id = find_user_id(&client, api_token.as_str(), user_email, user_name)
            .await?;

        // Открываем канал для сообщений
        let channel_id = messaging::open_direct_message_channel(&client, &api_token, &id)
            .await?;
        //println!("{}", channel_id);

        channel_id
    }else{
        return Err(Box::from("Missing channel, user email or user name"));
    };

    // Пишем текст
    if !text.is_empty() {
        // String можно преобразовать в &String, затем вызовется as_ref() -> &str
        messaging::send_direct_message_to_channel(&client, &api_token, &channel_id, text)
            .await?;
    }

    // Пишем QR
    if !qr_text.is_empty() {
        messaging::send_qr_to_channel(&client, &api_token, &channel_id, qr_text, qr_commentary)
            .await?;
    }

    Ok(())
}