
pub async fn send_message_to_channel(client: &reqwest::Client, api_token: &str, channel: &str, text: &str) -> Result<(), reqwest::Error>{
    // Выполняем POST запрос
    let post_params = vec![
        ("channel", channel),
        ("text", text)
    ];
    client.post("https://slack.com/api/chat.postMessage")
        .bearer_auth(api_token)
        .form(&post_params)
        .send()
        .await?;
    //println!("{:?}", response);
    
    Ok(())
}