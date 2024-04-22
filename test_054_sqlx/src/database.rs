use std::{
    path::{
        Path
    }
};
use tokio::{
    fs
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
        
        let db = Database{
            connection
        };

        Ok(db)
    }    
}