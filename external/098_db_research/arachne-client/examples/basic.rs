use arachne_client::{ArachneClient, User, UserId};
use chrono::Utc;
use serde_json::json;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Tarantool at 127.0.0.1:50051...");
    let client = ArachneClient::new("127.0.0.1:50051", 1).await?;

    // 1. Create a platform
    let platform_id = 1;
    println!("-> platform_create({})", platform_id);
    let instant = Instant::now();
    let p_status = client.platform_create(platform_id).await?;
    println!("Platform status: {:?}", p_status);
    println!("{:?}", instant.elapsed());

    println!("{:?}", client.total_users_count().await?);
    // 2. Create a selector
    let selector_id = 15;
    let query = "enabled == true";
    let s_status = client
        .selector_create(platform_id, selector_id, query, 60)
        .await?;
    println!(
        "-> selector_create(platform_id={}, selector_id={}, status={:?}, '{}')",
        platform_id, selector_id, s_status, query,
    );

    // 3. Add a user
    let user_payload = User {
        id: UserId::Int(2003),
        payload: r#"{"status": "waiting"}"#.to_string().into_bytes(),
        meta: json!({
            "enabled": true,
            "last_check": (Utc::now().timestamp() as u64 - 1000),
            "next_check_time": (Utc::now().timestamp() as u64 - 10)
        }),
    };

    let instant = Instant::now();

    println!(
        "-> user_add(user_id=2003) = {:?}",
        client.user_add(platform_id, user_payload).await?
    );

    println!("elapsed: {:?}", instant.elapsed());

    println!("Selector status: {:?}", s_status);
    // 4. Wait for Time Ticker to route
    println!("Waiting 1 second for the orchestrator to route the user...");
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    println!(
        "{:?}",
        client.total_platformed_users_count(platform_id).await?
    );
    // 5. Select the task
    println!(
        "-> select_user(platform_id={}, selector_id={})",
        platform_id, selector_id
    );
    println!(
        "queue size {:?}",
        client.selector_queue_size(platform_id, selector_id).await?
    );

    for _ in 0..3 {
        let instant = Instant::now();

        let task = client
            .select_user::<serde_json::Value>(platform_id, selector_id)
            .await?;

        println!("elapsed: {:?}", instant.elapsed());

        if let arachne_client::SelectUser::Found(t) = task {
            println!(
                "Got task for spider: ID = {}, payload = {:?}, meta = {:?}",
                t.id,
                String::from_utf8(t.payload).unwrap(),
                t.meta
            );
        } else {
            println!("No task returned. Status: {:?}", task);
        }
    }

    Ok(())
}
