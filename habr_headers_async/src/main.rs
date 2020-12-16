mod article;
mod client;
mod error;
mod page;
mod workers_pool;
mod load_save;
// mod print_support;

use crate::{article::HabrArticle, client::HabrClient, load_save::LoaderSaver};
use futures::future::{join, join_all};
use std::collections::hash_set::HashSet;
use tokio::{fs::File, runtime::Builder};

#[cfg_attr(feature = "flame_it", flamer::flame)]
async fn request_articles() -> Vec<HabrArticle> {
    let http_client = reqwest::Client::default();

    let client = HabrClient::new(http_client);

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
    let loader_saver = LoaderSaver::new(".habrahabr_headers.json");

    // Одновременно грузим с сервера ссылки + читаем прошлые ссылки из файлика
    let (results, previous_results) = {
        let articles_future = request_articles();
        let previous_future = loader_saver.load_previous_results();
        
        join(articles_future, previous_future)
            .await
    };

    println!("{:?}", results);

    // Запускаем одновременный вывод результата + сохранение результата
    // join(print_results(&selected, previous), save_links_to_file(&selected));
}

fn main() {
    let mut runtime = Builder::default()
        .enable_io()
        .threaded_scheduler()
        .core_threads(1)
        .max_threads(num_cpus::get())
        .build()
        .expect("Tokio runctime create failed");

    runtime.block_on(async_main());

    // Dump the report to disk
    #[cfg(feature = "flame_it")]
    flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
}
