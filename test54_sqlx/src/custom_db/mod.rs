use std::{
    path::{
        Path
    }
};
use futures::{
    StreamExt
};
use tokio::{
    runtime::{
        Builder
    }
};
use sqlx::{
    sqlite::{
        SqliteConnection,
        // SqlitePool
    },
    Connection,
};
use crate::{
    database::{
        Database
    }
};


async fn build_test_db_structure_if_needed(connection: &mut SqliteConnection) -> Result<(), sqlx::Error>{
    let need_create = {
        let mut stream = sqlx::query(include_str!("sql/check_db.sql"))
            .fetch(&mut (*connection));

        stream.next().await.is_none()
    };

    if need_create {
        //let sql = r#""#;
        let sql = include_str!("sql/create_db.sql");
        sqlx::query(sql)
            .execute(connection)
            .await?;
        println!("New database structure created");
    }

    Ok(())
}

pub async fn test_custom_db(){
    let db = Database::open(Path::new("database.sqlite"))
        .await
        .expect("Database open failed");

    let mut db: SqliteConnection = db.into();
    build_test_db_structure_if_needed(&mut db)
        .await
        .expect("New table structure failed");
}