mod article;
mod client;
mod error;
mod page;
mod workers_pool;

use crate::{article::HabrArticle, client::HabrClient, workers_pool::WorkersPool};
use futures::future::join_all;
use tokio::runtime::Builder;


#[cfg_attr(feature = "flame_it", flamer::flame)]
async fn request_articles(pool: WorkersPool) -> Vec<HabrArticle> {
    let http_client = reqwest::Client::default();

    let client = HabrClient::new(http_client, pool);

    const LINKS: [&str; 2] = [
        "https://habr.com/ru/all/",
        "https://habr.com/ru/all/page2/",
        //"https://habr.com/ru/all/page3/",
        //"https://habr.com/ru/all/page4/",
    ];

    let futures_iter = LINKS.iter().map(|url| client.request_page(url));

    let results: Vec<HabrArticle> = join_all(futures_iter)
        .await
        .into_iter()
        .filter_map(|val| match val {
            Ok(page) => Some(page),
            Err(err) => {
                print!("Page request error: {:?}", err);
                None
            }
        })
        .flat_map(|page| page.into_articles().into_iter())
        .collect();

    results
}

#[cfg_attr(feature = "flame_it", flamer::flame)]
async fn async_main() {
    let pool = WorkersPool::new(num_cpus::get());
    let results: Vec<HabrArticle> = request_articles(pool).await;

    println!("Articles: {:?}", results);
}

fn main() {
    let mut runtime = Builder::default()
        .enable_io()
        .basic_scheduler()
        .build()
        .expect("Tokio runctime create failed");

    runtime.block_on(async_main());

    // Dump the report to disk
    #[cfg(feature = "flame_it")]
    flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
}
