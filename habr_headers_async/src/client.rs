use tokio::{
    task::{
        spawn_blocking
    }
};
use reqwest::{
    Client
};
use super::{
    error::{
        HabrError
    },
    page::{
        HabrPage
    }
};


pub struct HabrClient{
    client: Client
}

impl HabrClient{
    pub fn new(client: Client) -> HabrClient{
        HabrClient{
            client
        }
    }
       
    async fn request_page(&self, link: &str) -> Result<HabrPage, HabrError>{
        let text = self.client
            .get(link)
            .send()
            .await?
            .text()
            .await?;

        let page = spawn_blocking(move || { HabrPage::parse_from(text) })
            .await
            .expect("Page parsing spawn failed");

        Ok(page)
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    // #[tokio::test]
    #[tokio::test(flavor = "multi_thread", worker_threads = 1)]
    async fn test_page_request(){
        let http_client = reqwest::Client::default();

        let client = HabrClient::new(http_client);

        let page = client
            .request_page("https://habr.com/ru/all/")
            .await
            .expect("Page request failed");

        let articles = page.get_articles();
        assert_eq!(articles.len() > 0, true);
    }
}