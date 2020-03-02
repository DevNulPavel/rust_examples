#![warn(clippy::all)]

use std::borrow::Borrow;
use async_std::task;
use surf;
use url::{
    ParseError, 
    Url
};
use html5ever::tokenizer::{
    BufferQueue, Tag, TagKind, TagToken, Token, TokenSink, TokenSinkResult, Tokenizer,
    TokenizerOpts,
};
use futures::channel::oneshot;
use futures::future::FutureExt;




type CrawlResult = Result<(), Box<dyn std::error::Error + Send + Sync + 'static>>;

type BoxFuture = std::pin::Pin<Box<dyn std::future::Future<Output = CrawlResult> + Send>>;

#[derive(Default, Debug)]
struct LinkQueue {
    links: Vec<String>,
}

impl TokenSink for &mut LinkQueue {
    type Handle = ();

    // <a href="link">some text</a>
    fn process_token(&mut self, token: Token, _: u64) -> TokenSinkResult<Self::Handle> {
        if let TagToken(tag) = token {
            if tag.kind == TagKind::StartTag && tag.name.as_ref() == "a" {
                for attribute in tag.attrs.iter() {
                    if attribute.name.local.as_ref() == "href" {
                        let url_str: &[u8] = attribute.value.borrow();
                        self.links
                            .push(String::from_utf8_lossy(url_str).into_owned());
                    }
                }
            }
        }
        /*match token {
            TagToken(
                ref tag @ Tag {
                    kind: TagKind::StartTag,
                    ..
                },
            ) => {

            }
            _ => {}
        }*/
        TokenSinkResult::Continue
    }
}

// Выдергиваем ссылки для страницы
fn get_links(url: &Url, page: String) -> Vec<Url> {
    let mut domain_url = url.clone();
    domain_url.set_path("");
    domain_url.set_query(None);

    let mut queue = LinkQueue::default();
    let mut tokenizer = Tokenizer::new(&mut queue, TokenizerOpts::default());
    let mut buffer = BufferQueue::new();
    buffer.push_back(page.into());
    let _ = tokenizer.feed(&mut buffer);

    queue
        .links
        .iter()
        .map(|link| match Url::parse(link) {
            Err(ParseError::RelativeUrlWithoutBase) => domain_url.join(link).unwrap(),
            Err(_) => panic!("Malformed link found: {}", link),
            Ok(url) => url,
        })
        .collect()
}

// Выполняем обход
async fn crawl(pages: Vec<Url>, current: u8, max: u8) -> CrawlResult {
    println!("Current Depth: {}, Max Depth: {}", current, max);

    // Если дошли до максимума глубины - выход
    if current > max {
        println!("Reached Max Depth");
        return Ok(());
    }

    // Пул потоков для асинхронных задач
    let pool = futures::executor::ThreadPool::new().unwrap();

    // Подзадачи обхода
    let mut tasks = vec![];

    println!("crawling: {:?}", pages);

    pages
        .into_iter() // Создаем потребляющий итератор
        .for_each(|url|{
            let pool = pool.clone();
            // Создаем новую асинхронную задачу
            let task = task::spawn(async move {
                println!("getting: {}", url);
                
                // Выполняем запрос
                let mut res = surf::get(&url).await?;

                // Получаем тело
                let body = res.body_string().await?;
                
                // Выдергиваем ссылки
                let (tx, rx) = oneshot::channel::<Vec<Url>>();
                pool.spawn_ok(async move {
                    let links = get_links(&url, body);
                    tx.send(links).unwrap();
                });
                let links = rx.await.unwrap();
                
                println!("Following: {:?}", links);

                // Обходим дочерние ссылки
                box_crawl(links, current + 1, max).await
            });
            tasks.push(task);
        });

    // Дожидаемся результата
    for task in tasks.into_iter() {
        task.await?;
    }

    Ok(())
}

// TODO: Для чего нужно оборачивать в бокс???
fn box_crawl(pages: Vec<Url>, current: u8, max: u8) -> BoxFuture {
    Box::pin(crawl(pages, current, max))
}

fn main() -> CrawlResult {
    // Создаем асинхронную задачу из std-async с блокировкой текущего потока на задаче
    task::block_on(async {
        let urls = vec![
            Url::parse("https://www.rust-lang.org").unwrap()
        ];
        // Начинаем обход
        //box_crawl(urls, 1, 2).await
        crawl(urls, 1, 2).await
    })
}
