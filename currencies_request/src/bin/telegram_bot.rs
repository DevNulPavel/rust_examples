use std::env;

use futures::StreamExt;
use telegram_bot::{
    Api,
    UpdatesStream,
    UpdateKind,
    MessageKind,
    Error,
    CanReplySendMessage,
    Update,
};

#[tokio::main(max_threads = 1)]
async fn main() -> Result<(), Error> {
    let token: String = env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN not set");
    println!("Token: {}", token);

    let api: Api = Api::new(token);

    // tracing::subscriber::set_global_default(
    //     tracing_subscriber::FmtSubscriber::builder()
    //         .with_env_filter("telegram_bot=trace")
    //         .finish(),
    // )
    // .unwrap();

    // Дергаем новые обновления через long poll метод
    let mut stream: UpdatesStream = api.stream();

    // Идем по новым событиям
    while let Some(update) = stream.next().await {
        // If the received update contains a new message...
        let update: Update = update?;

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