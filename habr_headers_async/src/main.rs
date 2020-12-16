mod article;
mod client;
mod error;
mod load_save;
mod page;
mod print_support;
// mod workers_pool;

use crate::{
    article::HabrArticle, client::HabrClient, load_save::LoaderSaver, print_support::print_results,
};
use futures::future::{join, join_all};
use tokio::runtime::Builder;

#[cfg_attr(feature = "flame_it", flamer::flame)]
async fn request_articles() -> Vec<HabrArticle> {
    let http_client = reqwest::Client::default();

    let client = HabrClient::new(http_client);

    const LINKS: &[&str] = &[
        "https://habr.com/ru/all/",
        "https://habr.com/ru/all/page2/",
        // "https://habr.com/ru/all/page3/",
        // "https://habr.com/ru/all/page4/",
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
    let loader_saver = LoaderSaver::new(".habrahabr_headers.json");

    // Одновременно грузим с сервера ссылки + читаем прошлые ссылки из файлика
    let (selected, previous) = {
        let articles_future = request_articles();
        let previous_future = loader_saver.load_previous_results();

        join(articles_future, previous_future).await
    };

    // Запускаем одновременный вывод результата + сохранение результата
    let print_future = print_results(&selected, previous);
    let save_future = loader_saver.save_links_to_file(&selected);
    join(print_future, save_future).await;
}

fn main() {
    let mut runtime = Builder::default()
        .enable_io()
        .basic_scheduler()
        //.threaded_scheduler()
        //.core_threads(1)
        //.max_threads(2)
        .build()
        .expect("Tokio runctime create failed");

    runtime.block_on(async_main());

    // Dump the report to disk
    #[cfg(feature = "flame_it")]
    flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
}
