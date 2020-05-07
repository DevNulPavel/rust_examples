#![warn(clippy::all)]

mod errors;
mod alpha;
mod types;

use tokio::{
    runtime::{
        Builder,
        Runtime
    }
};
use prettytable::{
    //color,
    Table, 
    Row, 
    Cell, 
    //Attr
};
use crate::{
    errors::CurrencyError,
    types::CurrencyResult,
    //types::CurrencyChange,
    alpha::get_currencies_from_alpha,
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


async fn async_main(){
    let result: Result<CurrencyResult, CurrencyError> = get_currencies_from_alpha("Alpha-Bank").await;

    let results = [
        result
    ];

    // Create the table
    let mut table = Table::new();

    table.add_row(Row::new(vec![
        new_cell!("Bank"),
        new_cell!("Buy USD"),
        new_cell!("Sell USD"),
        new_cell!("Buy EUR"),
        new_cell!("Sell EUR"),
        new_cell!("Update time")
    ]));    

    // Выводим текст, используем into_iter для потребляющего итератора
    for info in results.iter() {
        let info: &Result<CurrencyResult, CurrencyError> = info;
        match info {
            Ok(info) =>{
                let info: &CurrencyResult = info;

                let time_str: String = match info.update_time {
                    Some(time) => time.format("%Y-%m-%d %H:%M:%S").to_string(),
                    None => "No time".into()
                };

                // TODO: Макрос
                let row = Row::new(vec![
                    new_cell!(info.bank_name),
                    new_cell!("{} {}", info.usd.buy, info.usd.buy_change),
                    new_cell!("{} {}", info.usd.sell, info.usd.sell_change),

                    new_cell!("{} {}", info.eur.buy, info.eur.buy_change),
                    new_cell!("{} {}", info.eur.sell, info.eur.sell_change),
                    new_cell!(time_str.as_str()),
                ]);

                table.add_row(row);    
            },
            Err(e) => {
                let row = Row::new(vec![
                    Cell::new(format!("{:?}", e).as_str()),
                ]);
                table.add_row(row);    
            }
        }
    }

    // Print the table to stdout
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