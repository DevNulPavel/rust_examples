#![warn(clippy::all)]
#![allow(dead_code)]

extern crate prettytable;
extern crate reqwest;
extern crate scraper;
extern crate serde;
extern crate serde_json;

use rayon::prelude::*;

use std::collections::HashSet;
use std::fmt::Formatter;
use std::fmt::Display;
use std::fmt::Debug;
use reqwest::blocking::get;
use scraper::Html;
use scraper::Selector;
// use prettytable::format;
use prettytable::color;
use prettytable::{Table, Row, Cell, Attr};
//use serde::{Serialize, Deserialize};
// use itertools::Itertools;


const CACHE_FILE_NAME: &str = "habrahabr_headers.json";

//////////////////////////////////////////////////////////////////////////////////////////////////////

macro_rules! ToCSSError {
    ($expression:expr) => {
        $expression.map_err(|err|{
            HabrErrors::CSSParseError(format!("{:?}", err))
        })
    };
}

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

//////////////////////////////////////////////////////////////////////////////////////////////////////

// #[derive(Debug)]
enum HabrErrors{
    RequestError(reqwest::Error),
    CSSParseError(String)
}
impl From<reqwest::Error> for HabrErrors{
    fn from(err: reqwest::Error) -> Self{
        Self::RequestError(err)
    }
}
impl Debug for HabrErrors{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RequestError(err) => write!(f, "{}", err),
            Self::CSSParseError(err) => write!(f, "{}", err),
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)] // Автоматический debug вывод
struct HabrTitle{
    time: String,
    tags: Vec<String>,
    title: String,
    link: String
}
impl Display for HabrTitle{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {} ({})", self.title, self.time, self.link)
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

struct CssSelectors{
    preview_selector: scraper::Selector,
    time_selector: scraper::Selector,
    tags_selector: scraper::Selector,
    link_selector: scraper::Selector,
}

//////////////////////////////////////////////////////////////////////////////////////////////////////

fn text_to_multiline(text: &str, words_count: usize, intent: Option<&str>) -> String{
    let mut line_words_count = 0;
    let mut multiline_text: String = text
        .split(' ')
        .map(|word|{
            if let Some(last) = word.chars().last(){
                if last == '\n'{
                    line_words_count = 0;
                }
            }

            line_words_count += 1;

            if line_words_count % words_count != 0 {
                vec![word, " "]
            }else if let Some(intent) = intent{
                vec![word, "\n", intent]
            }else{
                vec![word, "\n"]
            }
        })
        .flatten()
        .collect();

    // Убираем пробел или \n в конце
    while let Some(last) = multiline_text.chars().last() {
        if last.is_whitespace(){
            multiline_text.pop();
        }else{
            break;
        }
    }
    
    multiline_text
}

fn print_results(selected: &[HabrTitle], previous_results: Option<HashSet<String>>){
    // Create the table
    let mut table = Table::new();

    // let format = format::FormatBuilder::new()
    //     .indent(0)
    //     .column_separator(' ')
    //     .borders(' ')
    //     .separators(&[format::LinePosition::Intern,
    //                 format::LinePosition::Bottom],
    //                 format::LineSeparator::new(' ', ' ', ' ', ' '))
    //     .padding(1, 1)
    //     .build();
    // table.set_format(format);

    // table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    // Выводим текст, используем into_iter для потребляющего итератора
    for info in selected.iter().rev() {
        // TODO: Может можно оптимальнее??

        let multiline_text = text_to_multiline(&info.title, 3, None);
        // multiline_text.push_str("\n\n");
        // multiline_text.push_str(&info.tags);

        let tags_text = text_to_multiline(&info.tags.join("\n"), 3, Some(" "));

        let text_color = previous_results
            .as_ref()
            .map(|set|{
                if set.contains(info.link.as_str()) {
                    color::GREEN
                }else{
                    color::YELLOW
                }
            })
            .unwrap_or(color::YELLOW);

        let row = Row::new(vec![
                Cell::new(&multiline_text)
                    .with_style(Attr::ForegroundColor(text_color)),
                Cell::new(&tags_text)
                    .with_style(Attr::ForegroundColor(color::WHITE)),
                Cell::new(&info.time)
                    .with_style(Attr::ForegroundColor(color::WHITE)),
                Cell::new(&info.link)
            ]);
        table.add_row(row);
    }

    // Print the table to stdout
    table.printstd();
}

fn request_links_from_page(link: &str, shared_selectors: &CssSelectors) -> Result<Vec<HabrTitle>, HabrErrors>{
    let page_text = get(link)?
        .text()?;

    // Парсим
    let parsed = Html::parse_document(&page_text);
    drop(page_text);

    // https://docs.rs/scraper/0.11.0/scraper/element_ref/struct.ElementRef.html
    let selected: Vec<HabrTitle> = parsed
        .select(&shared_selectors.preview_selector)
        .map(|preview_element|{
            let time = preview_element.select(&shared_selectors.time_selector).take(1).next();
            let link = preview_element.select(&shared_selectors.link_selector).take(1).next();
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

            // TODO: Может быть можно улучшить???
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
            
            //let tags_str = tags.join("\n");

            HabrTitle{
                time,
                tags,
                title: text,
                link: href
            }
        })
        .collect();

    Ok(selected)    
}

fn receive_habr_info() -> Vec<HabrTitle>{
    const LINKS: [&str; 4] = [
        "https://habr.com/ru/all/",
        "https://habr.com/ru/all/page2/",
        "https://habr.com/ru/all/page3/",
        "https://habr.com/ru/all/page4/",
    ];

    // Создаем селекторы по классу заранее
    let selectors = CssSelectors{
        preview_selector: ToCSSError!(Selector::parse(".post.post_preview")).unwrap(),
        time_selector: ToCSSError!(Selector::parse(".post__time")).unwrap(),
        tags_selector: ToCSSError!(Selector::parse(".inline-list__item-link.hub-link")).unwrap(),
        link_selector: ToCSSError!(Selector::parse(".post__title_link")).unwrap(),
    };

    // Обходим параллельно все ссылки
    let selected: Vec<HabrTitle> = LINKS
        .par_iter()
        // Код ниже исполняется параллельно
        .map(|link: &&str| -> Result<Vec<HabrTitle>, HabrErrors> {
            request_links_from_page(*link, &selectors)
        })
        // Фильтруем проблемные
        .filter(|value|{
            if value.is_err(){
                eprintln!("{:?}", value);
                return false;
            }
            true
        })
        // Отбрасываем ошибку
        .map(|value|{
            value.unwrap()
        })
        // Превращаем в общий массив
        .flatten()
        // Собираем в кучу
        .collect();
    
    selected
}

fn preload_previous_results() -> Option<HashSet<String>> {
    let temp_folder_path = std::env::temp_dir();
    let cache_file_path = std::path::PathBuf::new()
        .join(temp_folder_path)
        .join(CACHE_FILE_NAME);

    let result: Option<HashSet<String>> = std::fs::File::open(cache_file_path)
        .ok() // Превращаем результат в опцию
        .and_then(|file|{
            serde_json::from_reader::<_, HashSet<String>>(file)
                .ok()
        });

    //println!("{:?}", result);
    result
}

fn save_links_to_file(links: &[HabrTitle]){
    let temp_folder_path = std::env::temp_dir();
    let cache_file_path = std::path::PathBuf::new()
        .join(temp_folder_path)
        .join(CACHE_FILE_NAME);

    let links_iter: Vec<&str> = links
        .iter()
        .map(|info|{
            info.link.as_str()
        })
        .collect();

    std::fs::File::create(cache_file_path)
        .ok()
        .and_then(|file|{
            serde_json::to_writer(file, &links_iter)
                .ok()
        });
}

fn main() -> Result<(), HabrErrors> {

    // Одновременно грузим с сервера ссылки + читаем прошлые ссылки из файлика
    let (selected, previous) = rayon::join(||{
        receive_habr_info()
    }, ||{
        preload_previous_results()
    });
    
    rayon::join(||{
        print_results(&selected, previous);
    }, ||{
        save_links_to_file(&selected);
    });

    Ok(())
}