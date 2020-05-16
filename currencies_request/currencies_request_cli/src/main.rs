#![warn(clippy::all)]

use std::{
    time::Duration
};
//use futures::FutureExt;
use tokio::{
    runtime::{
        Builder,
        Runtime
    }
};
use reqwest::{
    Client,
    ClientBuilder,
};
use prettytable::{
    color::{
        self,
        Color
    },
    Table, 
    Row, 
    Cell, 
    Attr
};
use currency_lib::{
    CurrencyError,
    CurrencyResult,
    CurrencyChange,
    get_all_currencies
};

// https://doc.rust-lang.org/rust-by-example/macros/designators.html
// https://doc.rust-lang.org/reference/macros-by-example.html
// item: an Item
// block: a BlockExpression
// stmt: a Statement without the trailing semicolon (except for item statements that require semicolons)
// pat: a Pattern
// expr: an Expression
// ty: a Type
// ident: an IDENTIFIER_OR_KEYWORD
// path: a TypePath style path
// tt: a TokenTree (a single token or tokens in matching delimiters (), [], or {})
// meta: an Attr, the contents of an attribute
// lifetime: a LIFETIME_TOKEN
// vis: a possibly empty Visibility qualifier
// literal: matches -?LiteralExpression
macro_rules! new_cell {
    ($param: expr) => {
        Cell::new($param)
    };
    ($format: expr, $($params: expr),+) => {
        Cell::new(format!($format, $($params),+).as_str())
    };
}

fn change_to_color(change: CurrencyChange) -> Color {
    match change {
        CurrencyChange::Increase => color::GREEN,
        CurrencyChange::Decrease => color::RED,
        CurrencyChange::NoChange => color::BRIGHT_YELLOW
    }
}

fn new_cell_with_color(val: f32, change: CurrencyChange) -> Cell {
    let color = change_to_color(change);
    Cell::new(format!("{} {}", val, change).as_str())
        .with_style(Attr::ForegroundColor(color))
}

async fn async_main(){
    // Создаем клиента для запроса
    let client: Client = ClientBuilder::new()
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(3))
        .build()
        .unwrap();

    // Таблица
    let mut table = Table::new();

    // Заголовок
    table.add_row(Row::new(vec![
        new_cell!("Bank"),
        new_cell!("Buy USD"),
        new_cell!("Sell USD"),
        new_cell!("Buy EUR"),
        new_cell!("Sell EUR"),
        new_cell!("Update time")
    ]));    

    // Выводим текст, используем into_iter для потребляющего итератора
    for info in get_all_currencies(&client).await {
        let info: Result<CurrencyResult, CurrencyError> = info;
        match info {
            Ok(info) =>{
                let info: CurrencyResult = info;

                let time_str: String = match info.update_time {
                    Some(time) => time.format("%H:%M %d-%m-%Y").to_string(),
                    None => "No time".into()
                };

                // Строка со значениями
                let row = Row::new(vec![
                    new_cell!(info.bank_name.as_str()),
                    new_cell_with_color(info.usd.buy, info.usd.buy_change),
                    new_cell_with_color(info.usd.sell, info.usd.sell_change),

                    new_cell_with_color(info.eur.buy, info.eur.buy_change),
                    new_cell_with_color(info.eur.sell, info.eur.sell_change),
                    new_cell!(time_str.as_str()),
                ]);

                table.add_row(row);    
            },
            Err(_e) => {
                // TODO: Вывод ошибок
                /*let row = Row::new(vec![
                    Cell::new(format!("{:?}", e).as_str()),
                ]);
                table.add_row(row);*/
                println!("{:?}", _e);
            }
        }
    }

    // Вывод таблицы
    table.printstd();
}

fn main() {
    // Создаем однопоточный рантайм, здесь нет нужды в многопоточном
    let mut runtime: Runtime = Builder::new()
        .basic_scheduler()
        .enable_all()
        .build()
        .unwrap();
    
    runtime.block_on(async_main());
}