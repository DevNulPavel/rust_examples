use std::fmt::Display;


// Описание интерфеса суммрования
pub trait Summarizable {
    fn summary(&self) -> String;
}

// Структура новой статьи
pub struct NewsArticle {
    pub headline: String,
    pub location: String,
    pub author: String,
    pub content: String,
}

// Структура нового твита
pub struct Tweet {
    pub username: String,
    pub content: String,
    pub reply: bool,
    pub retweet: bool,
}

// Реализация интерфейса суммирования для новой статьи
impl Summarizable for NewsArticle {
    fn summary(&self) -> String {
        format!("{}, by {} ({})", self.headline, self.author, self.location)
    }
}

// Реализация интерфейса суммирования для нового сообщения
impl Summarizable for Tweet {
    fn summary(&self) -> String {
        format!("{}: {}", self.username, self.content)
    }
}

fn interface_test(){
    let tweet = Tweet {
        username: String::from("horse_ebooks"),
        content: String::from("of course, as you probably already know, people"),
        reply: false,
        retweet: false,
    };
    println!("1 new tweet: {}", tweet.summary());
}

// Шаблонная структура пары
struct Pair<T> {
    x: T,
    y: T,
}

// Реализация методов шаблонного типа пары
impl<T> Pair<T> {
    // Создание нового экземпляра
    fn new(x: T, y: T) -> Self {
        Self {
            x,
            y,
        }
    }
}

// Реализация интерфейсов Display + PartialOrd
impl<T: Display + PartialOrd> Pair<T> {
    fn cmp_display(&self) {
        if self.x >= self.y {
            println!("The largest member is x = {}", self.x);
        } else {
            println!("The largest member is y = {}", self.y);
        }
    }
}

fn main(){
    interface_test();
}