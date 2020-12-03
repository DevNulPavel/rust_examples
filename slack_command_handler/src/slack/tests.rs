use reqwest::{
    Client
};
use super::{
    client::{
        SlackClient,
        SlackMessageTaget,
        SlackImageTaget
    }, 
    // message::{
    //     Message
    // }
};
use crate::{
    qr::{
        create_qr_data,
        QrCodeError
    }
};

fn setup_logs() {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info,slack_command_handler=trace");
    env_logger::init();
}

fn build_client() -> SlackClient{
    // Slack api token
    let slack_api_token = std::env::var("SLACK_API_TOKEN")
        .expect("SLACK_API_TOKEN environment variable is missing");

    let client = SlackClient::new(Client::new(), &slack_api_token);

    client
}

#[actix_rt::test]
async fn test_direct_message() {
    //setup_logs();

    let client = build_client();

    client
        .send_message("Test message", SlackMessageTaget::to_user_direct("U0JU3ACSJ"))
        .await
        .expect("Direct message failed");

    let formatted_text = format!("*Jenkins bot error:*```{}```", "TEST");
    let mut message = client
        .send_message(&formatted_text, SlackMessageTaget::to_user_direct("U0JU3ACSJ"))
        .await
        .expect("Formatted direct message failed")
        .expect("Direct message - message object does not exist");

    actix_web::rt::time::delay_for(std::time::Duration::from_secs(2))
        .await;
    
    message
        .update_text("New text")
        .await
        .expect("Direct message update failed");

    let mut message = client
        .send_message("Test message", SlackMessageTaget::to_channel("#mur-test_node_upload"))
        .await
        .expect("Channel message failed")
        .expect("Channel message - message object does not exist");

    message
        .update_text("New text")
        .await
        .expect("Channel message update failed");

    client
        .send_message("Test message", SlackMessageTaget::to_thread("#mur-test_node_upload", message.get_timestamp()))
        .await
        .expect("Thread message failed")
        .expect("Thread message object get failed");

    let message = client
        .send_message("Test message", SlackMessageTaget::to_channel_ephemeral("#mur-test_node_upload", "U0JU3ACSJ"))
        .await
        .expect("Ephemeral message failed");

    assert_eq!(message.is_none(), true);

    // TODO: RESPONSE URL может фейлиться, не протестировано
}


#[actix_rt::test]
async fn test_image_upload() {
    setup_logs();

    let client = build_client();

    let image_data = create_qr_data("This is test text")
        .expect("Qr code create failed");

    let message = client
        .send_message("Test message", SlackMessageTaget::to_channel("#mur-test_node_upload"))
        .await
        .expect("Channel message failed")
        .expect("Channel message - message object does not exist");

    client.send_image(image_data, "Test text", "Test commentary", SlackImageTaget::to_thread(message.get_channel_id(), message.get_timestamp()))
        .await
        .expect("Image send failed");
}

/*#[actix_rt::test]
async fn test_open_view() {
    setup_logs();

    let client = build_client();

    let window_view = {}

    let window = serde_json::json!({
        "trigger_id": session.base.trigger_id,
        "view": window_view
    });
    
    let open_result = session
        .base
        .app_data
        .slack_client
        .open_view(window)
        .await;
}*/