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
    },
    Connection,
};

pub struct Database{
    connection: SqliteConnection
}

impl Into<SqliteConnection> for Database{
    fn into(self) -> SqliteConnection {
        self.connection
    }
}

impl Database{
    pub async fn open(path: &Path) -> Result<Database, sqlx::Error> {
        if path.exists() == false{
            fs::File::create(path)
                .await
                .map_err(|err|{
                    sqlx::error::Error::Io(err)
                })?;
        }

        let connection = {
            let path_str = path
            .to_str().expect("DB path as string failed");
            SqliteConnection::connect(path_str)
                .await?
        };
        
        let mut db = Database{
            connection
        };

        db
            .build_structure_if_needed()
            .await?;

        Ok(db)
    }

    async fn build_structure_if_needed(&mut self) -> Result<(), sqlx::Error>{
        let need_create = {
            let mut stream = sqlx::query(include_str!("sql/check_db.sql"))
                .fetch(&mut self.connection);

            stream.next().await.is_none()
        };

        if need_create {
            //let sql = r#""#;
            let sql = include_str!("sql/create_db.sql");
            sqlx::query(sql)
                .execute(&mut self.connection)
                .await?;
            println!("New database structure created");
        }

        Ok(())
    }
}