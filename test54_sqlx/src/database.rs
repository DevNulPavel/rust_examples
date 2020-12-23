use sqlx::{
    sqlite::{
        SqliteConnection
    },
    Row,
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
    pub async fn open(path: &str) -> Result<Database, sqlx::Error> {
        let connection = SqliteConnection::connect(path)
            .await?;
        
        let mut db = Database{
            connection
        };

        db
            .build_structure_if_needed()
            .await?;

        Ok(db)
    }

    async fn build_structure_if_needed(&mut self) -> Result<(), sqlx::Error>{
        let result = sqlx::query("SELECT tables FROM master")
            .fetch_all(&mut self.connection)
            .await?;

        result
            .iter()
            .for_each(|row|{
                //let colums = row.columns();
                let name: String = row.get("name");
                println!("Create result: {}", name);
            });

        Ok(())
    }
}