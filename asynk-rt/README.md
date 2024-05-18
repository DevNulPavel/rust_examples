# asynk
Rust multithread asynchronous runtime and reactor

## Example
```rust
use futures::future;
use futures_timer::Delay;
use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

fn main() {
    asynk::builder().build().unwrap();
    asynk::block_on(main_future()).unwrap();
}

async fn main_future() {
    let val = Arc::new(AtomicU32::new(0));
    let expected_val = 10_000;

    let handles = (0..expected_val)
        .map(|_| Arc::clone(&val))
        .map(|val| {
            asynk::spawn(async move {
                // some computations ...
                Delay::new(Duration::from_secs(1)).await;
                val.fetch_add(1, Ordering::SeqCst);
            })
        })
        .collect::<Vec<_>>();

    future::join_all(handles).await;

    assert_eq!(val.load(Ordering::SeqCst), expected_val);
}
```

# asynk-hyper

[Hyper](https://github.com/hyperium/hyper) integration with `asynk` runtime

## Example

```rust
use asynk_hyper::TcpListener;
use futures::StreamExt;
use http_body_util::Full;
use hyper::{body::Bytes, server::conn::http1, service::service_fn, Request, Response};
use std::convert::Infallible;

const SERVER_SOCK_ADDR: &str = "127.0.0.1:8040";

fn main() {
    asynk::builder().build().unwrap();
    asynk::block_on(server()).unwrap();
}

async fn server() {
    let addr = SERVER_SOCK_ADDR.parse().unwrap();

    let listener = TcpListener::bind(addr).unwrap();
    let mut accept = listener.accept();

    while let Some(res) = accept.next().await {
        // Spawn new task for the connection
        asynk::spawn(async move {
            // Accept the connection
            let (stream, _) = res.unwrap();

            if let Err(e) = http1::Builder::new()
                .serve_connection(stream, service_fn(hello))
                .await
            {
                eprintln!("error serving connection: {:?}", e);
            }
        });
    }
}

async fn hello(_: Request<impl hyper::body::Body>) -> Result<Response<Full<Bytes>>, Infallible> {
    Ok(Response::new(Full::new(Bytes::from(
        "<h1>Hello, World!</h1>",
    ))))
}

```