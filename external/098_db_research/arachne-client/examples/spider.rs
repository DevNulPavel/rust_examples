use arachne_client::{ArachneClient, User, UserId};
use chrono::{DateTime, Utc};
use isolang::Language;
use serde::{Deserialize, Serialize};
use std::time::Instant;

bitflags::bitflags! {
    /// Структура битовых масок
    #[derive(
        Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize
    )]
    struct Flags: u8 {
        const ENABLED = 0b00000001;
        const IMPORTANT = 0b00000010;
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[repr(Rust, packed)]
pub struct LanguageEntry {
    /// Язык, который может быть unknown в том числе.
    pub language: Language,

    /// Время детектирования опциональное, имеет смысл только если `language` не `Unknown`.
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub lang_detection_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserMeta {
    #[serde(with = "chrono::serde::ts_seconds_option")]
    last_check_time: Option<DateTime<Utc>>,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    last_activity_time: Option<DateTime<Utc>>,
    flags: Flags,
    subscribers_count: u32,
    #[serde(default)]
    primary_languages: [Option<LanguageEntry>; 4],
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Tarantool at 127.0.0.1:50051...");
    let client = ArachneClient::new("127.0.0.1:50051", 1).await?;

    // 1. Create a platform
    let platform_id = 255;
    println!("-> platform_create({})", platform_id);
    let instant = Instant::now();
    let p_status = client.platform_create(platform_id).await?;
    println!("Platform status: {:?}", p_status);
    println!("{:?}", instant.elapsed());

    println!("{:?}", client.total_users_count().await?);
    // 2. Create a selector
    let selector_id = 1;
    let query = format!(
        "flags CONTAINS \"ENABLED\" AND flags NOT CONTAINS \"IMPORTANT\" AND primary_languages ANY (language AMONG [\"rus\", \"eng\"] AND lang_detection_time AGE < 1 minute)"
    );
    let s_status = client
        .selector_create(platform_id, selector_id, &query, 60)
        .await?;
    println!(
        "-> selector_create(platform_id={}, selector_id={}, status={:?}, '{}')",
        platform_id, selector_id, s_status, query,
    );

    println!(
        "{}",
        serde_json::to_string_pretty(&UserMeta {
            last_check_time: None,
            last_activity_time: None,
            flags: Flags::ENABLED,
            subscribers_count: 0,
            primary_languages: [
                Some(LanguageEntry {
                    language: isolang::Language::from_name("Russian").unwrap(),
                    lang_detection_time: None
                }),
                None,
                None,
                None
            ]
        })
        .unwrap()
    );
    // 3. Add a user
    let user_payload = User {
        id: UserId::Int(1),
        payload: r#"{"status": "waiting"}"#.to_string().into_bytes(),
        meta: UserMeta {
            last_check_time: None,
            last_activity_time: None,
            flags: Flags::ENABLED,
            subscribers_count: 0,
            primary_languages: [
                Some(LanguageEntry {
                    language: isolang::Language::from_name("Russian").unwrap(),
                    lang_detection_time: Some(Utc::now()),
                }),
                None,
                None,
                None,
            ],
        },
    };

    let instant = Instant::now();

    println!(
        "-> user_add(user_id=1) = {:?}",
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

    Ok(())
}
