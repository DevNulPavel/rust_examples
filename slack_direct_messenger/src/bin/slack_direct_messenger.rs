#![warn(clippy::all)]
#![allow(dead_code)]

// Можно создавать отдельно библиотеку и отдельно код бинарника в папке bin
// В зависимости от имени файлика - создается исполняемый бинарник

use clap::{
    Arg, 
    App
};
// Так мы подключаем библиотеку для использования
use slack_direct_messenger::{
    messaging,
    user_search
};

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

    // Ищем id сначала по email, если не вышло - по имени
    let id = match user_search::find_user_id_by_email(&client, &api_token, email).await {
        Ok(id) => id,
        Err(_)=>{
            match user_search::find_user_id_by_name(&client, &api_token, user).await {
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
    let channel_id = messaging::open_direct_message_channel(&client, &api_token, &id).await?;
    //println!("{}", channel_id);
    
    if !text.is_empty() {
        // String можно преобразовать в &String, затем вызовется as_ref() -> &str
        messaging::send_direct_message_to_channel(&client, &api_token, &channel_id, text).await?;
    }

    if !qr_text.is_empty() {
        messaging::send_qr_to_channel(&client, &api_token, &channel_id, qr_text, qr_commentary).await?;
    }

    Ok(())
}