use dynomite::{retry::Policy, Attributes, DynamoDbExt, Item, Retries};
use eyre::WrapErr;
use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{
    AttributeDefinition, CreateTableError, CreateTableInput, DynamoDb, DynamoDbClient, KeySchemaElement,
    ListTablesInput, ProvisionedThroughput, PutItemInput,
};
use tracing::{debug, error, info, instrument, warn};
use uuid::Uuid;

////////////////////////////////////////////////////////////////////////////////////////////////////////////

// https://lessis.me/dynomite/dynomite/index.html
// #[dynomite(flatten)]
// #[dynomite(rename = "book_title")]
// #[dynomite(sort_key)]
// #[dynomite(partition_key)]
// #[dynomite(default)]

#[derive(Attributes, Debug, Clone)]
pub struct Author {
    id: Uuid,
    #[dynomite(default)]
    name: String,
}

#[derive(Item, Debug, Clone)]
pub struct Book {
    // Уникальный идентификатор
    #[dynomite(partition_key)]
    id: Uuid,
    // Имя книги
    #[dynomite(rename = "book_title")]
    title: String,
    // Список авторов
    authors: Option<Vec<Author>>,
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(client))]
async fn print_tables_list(client: &impl DynamoDb) -> Result<(), eyre::Error> {
    // Комманда для списка таблиц
    let list_tables_input: ListTablesInput = Default::default();

    // Делаем запрос
    let output = client
        .list_tables(list_tables_input)
        .await
        .context("Tables list request")?;

    // Выводим имеющиеся таблицы
    match output.table_names {
        Some(table_name_list) => {
            info!(tables = ?table_name_list, "Tables in database");
        }
        None => {
            info!("No tables in database!")
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(client))]
async fn create_table_if_needed(client: &impl DynamoDb, table_name: String) -> Result<(), eyre::Error> {
    // Parameters
    let input = CreateTableInput {
        table_name,
        key_schema: vec![KeySchemaElement {
            attribute_name: "id".to_string(),
            key_type: "HASH".to_string(),
        }],
        attribute_definitions: vec![AttributeDefinition {
            attribute_name: "id".to_string(),
            attribute_type: "S".to_string(),
        }],
        provisioned_throughput: Some(ProvisionedThroughput {
            read_capacity_units: 1,
            write_capacity_units: 1,
        }),
        ..Default::default()
    };

    // Создаем
    let res = client.create_table(input).await;

    // Смотрим результат
    match res {
        Ok(result) => {
            info!(?result, "Table created");
        }
        Err(RusotoError::Service(CreateTableError::ResourceInUse(err))) => {
            debug!(%err, "Table already exists, creating is not required");
        }
        Err(err) => {
            return Err(err).wrap_err("Table create");
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(client))]
async fn insert_book(client: &impl DynamoDb, table_name: String, book: Book) -> Result<(), eyre::Error> {
    let input = PutItemInput {
        table_name,
        item: book.into(),
        ..Default::default()
    };

    let result = client.put_item(input).await?;

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument]
pub async fn test_dynamo_db() -> Result<(), eyre::Error> {
    // Создаем клиента с определенным регионом
    /*let client = DynamoDbClient::new(Region::Custom{
        name: "eu-east-2".to_string(),
        endpoint: "http://localhost:8000".into()
    });*/
    let client = DynamoDbClient::new(Region::EuWest2);

    // Выводим таблички
    print_tables_list(&client).await?;

    // Создаем табличку
    create_table_if_needed(&client, "books".to_owned()).await?;

    // Добавляем книжку
    let book = Book {
        id: Uuid::new_v4(),
        title: "first_book_title".to_string(),
        authors: Some(vec![Author {
            id: Uuid::new_v4(),
            name: "Book author title".to_string(),
        }]),
    };
    insert_book(&client, "books".to_owned(), book).await?;

    Ok(())
}
