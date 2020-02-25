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


//type CSSErr = scraper::cssparser::ParseError<'i, SelectorParseErrorKind<'i>>;
macro_rules! ToCSSError {
    ($expression:expr) => {
        $expression.map_err(|err|{
            HabrErrors::CSSParseError(format!("{:?}", err))
        })
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
    title: String,
    link: String
}
impl Display for HabrTitle{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}, {} ({})", self.title, self.time, self.link)
    }
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
        let mut multiline_text: String = info.title
            .split(' ')
            .enumerate()
            .map(|(i, word)|{
                if (i+1) % 5 != 0 {
                    vec![word, " "]
                }else{
                    vec![word, "\n"]
                }
            })
            .flatten()
            .collect();
        multiline_text.pop();

        let text_color = previous_results
            .as_ref()
            .map(|set|{
                if set.contains(info.link.as_str()) {
                    color::YELLOW
                }else{
                    color::GREEN
                }
            })
            .unwrap_or(color::GREEN);

        let row = Row::new(vec![
                Cell::new(&multiline_text)
                    .with_style(Attr::ForegroundColor(text_color)),
                Cell::new(&info.time)
                    .with_style(Attr::ForegroundColor(color::WHITE)),
                Cell::new(&info.link)
            ]);
        table.add_row(row);
    }

    // Print the table to stdout
    table.printstd();
}

fn request_links(link: &str) -> Result<Vec<HabrTitle>, HabrErrors>{
    let page_text = get(link)?
        .text()?;

    // Парсим
    let parsed = Html::parse_document(&page_text);
    drop(page_text);

    // Создаем селектор по классу
    let preview_selector = ToCSSError!(Selector::parse(".post.post_preview"))?;
    let time_selector = ToCSSError!(Selector::parse(".post__time"))?;
    let link_selector = ToCSSError!(Selector::parse(".post__title_link"))?;

    // https://docs.rs/scraper/0.11.0/scraper/element_ref/struct.ElementRef.html
    let selected: Vec<HabrTitle> = parsed
        .select(&preview_selector)
        .map(|preview_element|{
            let time = preview_element.select(&time_selector).take(1).next();
            let link = preview_element.select(&link_selector).take(1).next();
            (time, link)
        })
        .filter(|(time, link)|{
            time.is_some() && link.is_some()
        })
        .map(|(time, link)|{
            (time.unwrap(), link.unwrap())
        })
        .map(|(time, link)|{
            // TODO: В макрос
            // Так как текст - это итератор, то нужно сначала создавать итератор, который может проверять следующий элемент
            // Затем пробовать этот элемент
            let time: Option<String> = match time.text().peekable().peek() {
                Some(_) => Some(time.text().collect()),
                None => None
            };

            // TODO: В макрос
            // Так как текст - это итератор, то нужно сначала создавать итератор, который может проверять следующий элемент
            // Затем пробовать этот элемент
            let text: Option<String> = match link.text().peekable().peek() {
                Some(_) => Some(link.text().collect()),
                None => None
            };

            let href = link.value().attr("href");

            (time, href, text)
        })
        .filter(|(time, href, text)|{
            let valid_time = time.is_some();
            let valid_text = text.is_some();
            let valid_href = href.is_some();
            valid_time && valid_href && valid_text
        })
        .map(|(time, href, text)|{
            let time = time.unwrap();
            let text = text.unwrap();
            let href = href.unwrap().to_owned();
            HabrTitle{
                time,
                title: text,
                link: href
            }
        })
        .collect();

    Ok(selected)    
}

fn receive_habr_info() -> Vec<HabrTitle>{
    const LINKS: [&str; 3] = [
        "https://habr.com/ru/all/",
        "https://habr.com/ru/all/page2/",
        "https://habr.com/ru/all/page3/"
    ];

    // Обходим параллельно все ссылки
    let selected: Vec<HabrTitle> = LINKS
        .par_iter()
        .map(|link: &&str| -> Result<Vec<HabrTitle>, HabrErrors> {
            request_links(*link)
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