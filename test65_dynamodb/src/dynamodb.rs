use dynomite::{attr_map, retry::Policy, Attribute, Attributes, DynamoDbExt, FromAttributes, Item, Retries};
use eyre::WrapErr;
use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{
    AttributeDefinition, CreateTableError, CreateTableInput, DescribeTableError, DescribeTableInput, DynamoDb,
    DynamoDbClient, GetItemInput, KeySchemaElement, ListTablesInput, ProvisionedThroughput, PutItemInput,
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

    // Вторичный ключ сортировки
    #[dynomite(sort_key)]
    year: u32,

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
        table_name: table_name.clone(),
        // Описание ключей
        key_schema: vec![
            KeySchemaElement {
                attribute_name: "id".to_string(),
                key_type: "HASH".to_string(), // Первичный ключ - HASH, ключ сортировки - RANGE
            },
            KeySchemaElement {
                attribute_name: "year".to_string(),
                key_type: "RANGE".to_string(), // Первичный ключ - HASH, ключ сортировки - RANGE
            },
        ],
        // Описание типов ключей
        attribute_definitions: vec![
            AttributeDefinition {
                attribute_name: "id".to_string(),
                attribute_type: "S".to_string(),
            },
            AttributeDefinition {
                attribute_name: "year".to_string(),
                attribute_type: "N".to_string(), // [B, N, S]
            },
        ],
        // Количество записей и чтения на отдельную таблицу в секунду
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
            info!("Table created");
            debug!(table_create_result = ?result);
        }
        Err(RusotoError::Service(CreateTableError::ResourceInUse(err))) => {
            debug!(%err, "Table already exists, creating is not required");
        }
        Err(err) => {
            return Err(err).wrap_err("Table create");
        }
    }

    // Дожидаемся когда таблица станет доступна для работы
    let mut check_count = 0;
    loop {
        // Получаем информацию о таблице
        let status = client
            .describe_table(DescribeTableInput {
                table_name: table_name.clone(),
                ..Default::default()
            })
            .await?
            .table
            .ok_or_else(||{
                eyre::eyre!("Table info request failed")
            })?
            .table_status
            .ok_or_else(||{
                eyre::eyre!("Table status is missing")
            })?;

        // Узнаем статус
        match status.as_str() {
            "ACTIVE" => {
                info!("Table is available now");
                break;                
            },
            status @ _ => {
                if check_count < 5 {
                    info!(status, "Table will be awailable soon, awaiting");
                    tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
                    check_count += 1;
                }else{
                    return Err(eyre::eyre!("Table available check timeout, status: {}", status));
                }
            }
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(client, book), fields(book_id = %book.id))]
async fn insert_book(client: &impl DynamoDb, table_name: String, book: Book) -> Result<(), eyre::Error> {
    let input = PutItemInput {
        table_name,
        item: book.into(),
        ..Default::default()
    };

    let result = client.put_item(input).await?;

    info!(book_result = ?result, "book insert success");

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(client))]
async fn get_book_with_title(
    client: &impl DynamoDb,
    table_name: String,
    book_key: Attributes,
) -> Result<Option<Book>, eyre::Error> {
    let result = client
        .get_item(GetItemInput {
            table_name,
            key: book_key,
            ..Default::default()
        })
        .await?
        .item;

    match result {
        Some(data) => {
            let book = Book::from_attrs(data)?;
            Ok(Some(book))
        }
        None => Ok(None),
    }
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

    let table_name = "books";

    // Выводим таблички
    print_tables_list(&client).await?;

    // Создаем табличку
    create_table_if_needed(&client, table_name.to_owned()).await?;

    // Добавляем книжку
    let book = Book {
        id: Uuid::new_v4(),
        year: 1999,
        title: "first_book_title".to_string(),
        authors: Some(vec![Author {
            id: Uuid::new_v4(),
            name: "Book author title".to_string(),
        }]),
    };
    let new_book_key = book.key();
    insert_book(&client, table_name.to_owned(), book).await?;

    // Пытаемся получить эту книжку
    let received_book = get_book_with_title(&client, table_name.to_owned(), new_book_key).await?;
    info!(?received_book, "Book received");

    Ok(())
}
