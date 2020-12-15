use scraper::{
    Html,
    Selector
};
use lazy_static::{
    lazy_static
};
use super::{
    article::{
        HabrArticle
    }
};

macro_rules! ToTextOption {
    ($expression:expr) => {
        // Так как текст - это итератор, то нужно сначала создавать итератор, который может проверять следующий элемент
        // Затем пробовать этот элемент
        match $expression.text().peekable().peek() {
            Some(_) => Some($expression.text().collect()),
            None => None
        }
    };
}

////////////////////////////////////////////////////////////////////////////////////////////////

struct CssSelectors{
    preview_selector: scraper::Selector,
    time_selector: scraper::Selector,
    tags_selector: scraper::Selector,
    link_selector: scraper::Selector,
}

////////////////////////////////////////////////////////////////////////////////////////////////

fn parse_links_from_page(text: String, shared_selectors: &CssSelectors) -> Vec<HabrArticle>{
    // Парсим
    let parsed = Html::parse_document(&text);
    drop(text);

    // https://docs.rs/scraper/0.11.0/scraper/element_ref/struct.ElementRef.html
    let selected: Vec<HabrArticle> = parsed
        .select(&shared_selectors.preview_selector)
        .map(|preview_element|{
            let time = preview_element
                .select(&shared_selectors.time_selector)
                .take(1)
                .next();
            let link = preview_element
                .select(&shared_selectors.link_selector)
                .take(1)
                .next();
            (time, link, preview_element)
        })
        .filter(|(time, link, _)|{
            time.is_some() && link.is_some()
        })
        .map(|(time, link, preview_element)|{
            (time.unwrap(), link.unwrap(), preview_element)
        })
        .map(|(time, link, preview_element)|{
            let time: Option<String> = ToTextOption!(time);
            let text: Option<String> = ToTextOption!(link);

            let href = link.value().attr("href");

            (time, href, text, preview_element)
        })
        .filter(|(time, href, text, _)|{
            time.is_some() && text.is_some() && href.is_some()
        })
        .map(|(time, href, text, preview_element)|{
            let time = time.unwrap();
            let text = text.unwrap();
            let href = href.unwrap().to_owned();

            // Выдергиваем теги
            let tags: Vec<String> = preview_element.select(&shared_selectors.tags_selector)
                .map(|element|{
                    let text: Option<String> = ToTextOption!(element);
                    text
                })
                .filter(|val|{
                    val.is_some()
                })
                .map(|val|{
                    format!("#{}", val.unwrap())
                })
                .collect();

            HabrArticle{
                time,
                tags,
                title: text,
                link: href
            }
        })
        .collect();

    selected
}

////////////////////////////////////////////////////////////////////////////////////////////////

pub struct HabrPage{
    articles: Vec<HabrArticle>
}

impl HabrPage{
    pub fn parse_from(content: String) -> HabrPage {
        // Создаем селекторы по классу заранее
        lazy_static! {
            static ref CSS_SELECTORS: CssSelectors = {
                CssSelectors{
                    preview_selector: Selector::parse(".post.post_preview").unwrap(),
                    time_selector: Selector::parse(".post__time").unwrap(),
                    tags_selector: Selector::parse(".inline-list__item-link.hub-link").unwrap(),
                    link_selector: Selector::parse(".post__title_link").unwrap(),
                }
            };
        }

        let articles = parse_links_from_page(content, &CSS_SELECTORS);

        HabrPage{
            articles
        }
    }

    pub fn get_articles(&self) -> &[HabrArticle]{
        self.articles.as_ref()
    }
}