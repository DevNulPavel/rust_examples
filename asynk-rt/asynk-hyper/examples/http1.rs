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
