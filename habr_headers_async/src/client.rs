// use tokio::{
//     task::{
//         spawn_blocking
//     }
// };
use super::{error::HabrError, page::HabrPage, workers_pool::WorkersPool};
use reqwest::Client;

pub struct HabrClient {
    client: Client,
    pool: WorkersPool,
}

impl HabrClient {
    pub fn new(client: Client, pool: WorkersPool) -> HabrClient {
        HabrClient { client, pool }
    }

    #[cfg_attr(feature = "flame_it", flamer::flame)]
    pub async fn request_page(&self, link: &str) -> Result<HabrPage, HabrError> {
        let text = self.client.get(link).send().await?.text().await?;

        let page = self
            .pool
            .queue_task(move || HabrPage::parse_from(text))
            .await;

        Ok(page)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_page_request() {
        let http_client = reqwest::Client::default();

        let pool = WorkersPool::new(1);

        let client = HabrClient::new(http_client, pool);

        let page = client
            .request_page("https://habr.com/ru/all/")
            .await
            .expect("Page request failed");

        let articles = page.into_articles();
        assert_eq!(articles.len() > 0, true);

        let article = articles.get(0).unwrap();
        assert_eq!(article.title.is_empty(), false);
    }
}
