use std::{
    path::{
        Path
    }
};
use tokio::{
    fs
};
use futures::{
    StreamExt
};
use sqlx::{
    sqlite::{
        SqliteConnection,
        SqliteRow
    },
    Connection,
    Row,
    ColumnIndex,
    Column,
    Type,
    TypeInfo
};
use crate::{
    database::{
        Database
    }
};

// TODO: Вполне нормально будт работать вариант с &impl Row
fn print_row_header<R>(row: &R) 
where R: Row {
    let header = row
        .columns()
        .iter()
        .fold(String::new(), |mut prev_str, col|{
            prev_str.push_str(col.name());
            prev_str.push_str(" ");
            prev_str
        });
    println!("{}", header);
}

fn print_row_values(row: &SqliteRow) {
    let text = (0..row.len())
        .fold(String::new(), |mut prev_str, i|{
            let type_info = row.column(i).type_info();

            match type_info.name(){
                "TEXT" => {
                    let val: String = row.get(i);
                    prev_str.push_str(&val);
                },
                "INTEGER" => {
                    let val: i64 = row.get(i);
                    prev_str.push_str(&format!("{}", val));
                },
                name @ _ => {
                    prev_str.push_str(&format!("Type: {}", name));        
                }
            }

            prev_str.push_str(" ");
            prev_str
        });
    println!("{}", text);
}

fn print_results(results: &Vec<SqliteRow>){
    results
        .iter()
        .take(1)
        .for_each(|row|{
            print_row_header(row);
        });
    
    results
        .iter()
        .for_each(|row|{
            print_row_values(row);
        })
}

pub async fn test_mm_db(){
    let db = Database::open(Path::new("test_databases/database.sqlite"))
        .await
        .expect("Database open failed");

    let mut db: SqliteConnection = db.into();

    {
        let levels = sqlx::query(include_str!("sql/select_levels.sql"))
            .fetch_all(&mut db)
            .await
            .expect("Levels select failed");

        print_results(&levels);
    }
}