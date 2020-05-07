use std::env;

use futures::StreamExt;
use telegram_bot::*;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let token = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
    println!("Token: {}", token);

    let api = Api::new(token);

    // Дергаем новые обновления через long poll метод
    let mut stream = api.stream();
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update = update?;

        println!("Update: {:?}", update);

        if let UpdateKind::Message(message) = update.kind {
            if let MessageKind::Text { ref data, .. } = message.kind {
                // Print received text message to stdout.
                println!("<{}>: {}", &message.from.first_name, data);

                // Answer message with "Hi".
                api.send(message.text_reply(format!(
                        "Hi, {}! You just wrote '{}'",
                        &message.from.first_name, data
                    )))
                    .await?;
            }
        }
    }
    Ok(())
}