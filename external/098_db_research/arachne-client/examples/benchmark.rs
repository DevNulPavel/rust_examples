use arachne_client::{ArachneClient, SelectUser, User, UserId};
use chrono::Utc;
use serde_json::{Value, json};
use smol_str::ToSmolStr;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Instant;
use tokio::task;

const NUM_USERS: usize = 1_000_000;
const CONCURRENCY: usize = 10;
const BATCH_SIZE: usize = NUM_USERS / CONCURRENCY;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Connecting to Tarantool...");
    let client = ArachneClient::new("127.0.0.1:50051", 2).await?;

    let platform_id = 1;
    client.platform_create(platform_id).await?;

    let selector_id = 10;

    let complex_query = "enabled == true AND (last_check AGE > 5 minutes OR retry_count < 3) AND (tags AMONG [\"important\", \"urgent\"])";
    let status = client
        .selector_create(platform_id, selector_id, complex_query, 60)
        .await?;
    println!("Selector create status: {:?}", status);

    println!(
        "\n--- Complex Benchmark: Insert {} users with {} tasks ---",
        NUM_USERS, CONCURRENCY
    );

    let start_time = Instant::now();
    let mut tasks = vec![];
    let inserted_counter = Arc::new(AtomicUsize::new(0));

    for i in 0..CONCURRENCY {
        let counter_clone = inserted_counter.clone();
        let worker_client = client.clone();
        let start_idx = i * BATCH_SIZE;
        let end_idx = (i + 1) * BATCH_SIZE;

        tasks.push(task::spawn(async move {
            for j in start_idx..end_idx {
                let payload = format!(r#"{{"some_data": "{}"}}"#, "x".repeat(500)).into_bytes();
                let user_payload = User {
                    id: UserId::Str(j.to_smolstr()),
                    payload,
                    meta: json!({
                        "enabled": true,
                        "last_check": (Utc::now().timestamp() as u64).saturating_sub(1000),
                        "retry_count": j % 5,
                        "tags": ["important", "news", "other"],
                        "next_check_time":  (Utc::now().timestamp() as u64).saturating_sub(10)
                    }),
                };
                let _ = worker_client.user_add(platform_id, user_payload).await;
            }
            counter_clone.fetch_add(end_idx - start_idx, Ordering::Relaxed);
        }));
    }

    futures::future::join_all(tasks).await;
    let insert_time = start_time.elapsed().as_secs_f64();
    let total_inserted = inserted_counter.load(Ordering::Relaxed);
    println!(
        "Insert {} took: {:.2}s ({:.0} ops/sec)",
        total_inserted,
        insert_time,
        total_inserted as f64 / insert_time
    );

    println!(
        "\n--- Select {} tasks with {} concurrency ---",
        NUM_USERS, CONCURRENCY
    );

    let start_time_select = Instant::now();
    let mut select_tasks = vec![];
    let selected_counter = Arc::new(AtomicUsize::new(0));
    let empty_counter = Arc::new(AtomicUsize::new(0));

    for _ in 0..CONCURRENCY {
        let selected_clone = selected_counter.clone();
        let empty_clone = empty_counter.clone();
        let worker_client = client.clone();

        select_tasks.push(task::spawn(async move {
            let mut local_found = 0;
            let mut local_empty = 0;

            for _ in 0..BATCH_SIZE {
                match worker_client
                    .select_user::<Value>(platform_id, selector_id)
                    .await
                {
                    Ok(SelectUser::Found(_)) => {
                        local_found += 1;
                    }
                    Ok(SelectUser::Empty) => {
                        local_empty += 1;
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    }
                    _ => {}
                }
            }

            selected_clone.fetch_add(local_found, Ordering::Relaxed);
            empty_clone.fetch_add(local_empty, Ordering::Relaxed);
        }));
    }

    futures::future::join_all(select_tasks).await;

    let select_time = start_time_select.elapsed().as_secs_f64();
    let total_selected = selected_counter.load(Ordering::Relaxed);
    let total_empty = empty_counter.load(Ordering::Relaxed);

    println!(
        "Select {} took: {:.2}s ({:.0} ops/sec)",
        total_selected + total_empty,
        select_time,
        (total_selected + total_empty) as f64 / select_time
    );
    println!(
        "(Found: {}, Empty responses: {})",
        total_selected, total_empty
    );
    Ok(())
}
