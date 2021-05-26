use dynomite::{attr_map, retry::Policy, Attribute, Attributes, DynamoDbExt, FromAttributes, Item, Retries};
use eyre::WrapErr;
use rusoto_core::{Region, RusotoError};
use rusoto_dynamodb::{
    AttributeDefinition, CreateTableError, CreateTableInput, DescribeTableError, DescribeTableInput, DynamoDb,
    DynamoDbClient, GetItemInput, GlobalSecondaryIndex, KeySchemaElement, ListTablesInput, LocalSecondaryIndex,
    Projection, ProvisionedThroughput, PutItemInput, QueryInput, UpdateItemInput,
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
    // id: Uuid,
    #[dynomite(default)]
    name: String,
}

#[derive(Item, Debug, Clone)]
pub struct Book {
    // Уникальный идентификатор
    // #[dynomite(partition_key)]
    // id: Uuid,

    // Вторичный ключ сортировки
    // #[dynomite(sort_key)]
    // year: u32,

    // Имя книги + уникальный идентификатор
    #[dynomite(partition_key, rename = "book_title")]
    title: String,

    book_year: u32,

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
                attribute_name: "book_title".to_string(),
                key_type: "HASH".to_string(), // Первичный ключ - HASH, ключ сортировки - RANGE
            },
            // KeySchemaElement {
            //     attribute_name: "year".to_string(),
            //     key_type: "RANGE".to_string(), // Первичный ключ - HASH, ключ сортировки - RANGE
            // },
        ],
        // Описание типов ключей
        attribute_definitions: vec![
            AttributeDefinition {
                attribute_name: "book_title".to_string(),
                attribute_type: "S".to_string(),
            },
            AttributeDefinition {
                attribute_name: "book_year".to_string(),
                attribute_type: "N".to_string(), // [B, N, S]
            },
        ],
        // Глобальные вторичные индексы нужны для поиска по главному ключу и по какому-то еще полю??
        global_secondary_indexes: Some(vec![GlobalSecondaryIndex {
            index_name: "year_index".to_owned(),
            key_schema: vec![KeySchemaElement {
                attribute_name: "book_year".to_owned(),
                key_type: "HASH".to_owned(),
            }],
            projection: Projection {
                projection_type: Some("ALL".to_owned()), // Можно указать лишь нужные нам поля из исходной таблицы
                ..Default::default()
            },
            provisioned_throughput: Some(ProvisionedThroughput {
                read_capacity_units: 1,
                write_capacity_units: 1,
            }),
            ..Default::default()
        }]),
        // Вторичные индексы нужны для поиска по главному ключу и по какому-то еще полю??
        // local_secondary_indexes: Some(vec![
        //     LocalSecondaryIndex{
        //         index_name: "year_index".to_owned(),
        //         key_schema: vec![KeySchemaElement{
        //             attribute_name: "year".to_owned(),
        //             key_type: "HASH".to_owned()
        //         }],
        //         ..Default::default()
        //     }
        // ]),
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
            debug!(%err, "Table already exists, creating is not required:");
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
            .ok_or_else(|| eyre::eyre!("Table info request failed"))?
            .table_status
            .ok_or_else(|| eyre::eyre!("Table status is missing"))?;

        // Узнаем статус
        match status.as_str() {
            "ACTIVE" => {
                info!("Table is available now");
                break;
            }
            status @ _ => {
                if check_count < 60 {
                    info!(status, "Table will be awailable soon, awaiting:");
                    tokio::time::delay_for(std::time::Duration::from_secs(1)).await;
                    check_count += 1;
                } else {
                    return Err(eyre::eyre!("Table available check timeout, status: {}", status));
                }
            }
        }
    }

    Ok(())
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(client, book), fields(book_title = %book.title))]
async fn insert_book(client: &impl DynamoDb, table_name: String, book: Book) -> Result<(), eyre::Error> {
    let input = PutItemInput {
        table_name,
        item: book.into(),
        ..Default::default()
    };

    let result = client.put_item(input).await?;

    info!(book_result = ?result, "book insert success:");

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
            // projection_expression: Some("title".to_owned()), // Выдергивание лишь определенных полей
            ..Default::default()
        })
        .await?
        .item;

    match result {
        Some(data) => {
            let book = Book::from_attrs(data).wrap_err("Book parsing")?;
            Ok(Some(book))
        }
        None => Ok(None),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(client))]
async fn query_books_with_year_greater(
    client: &impl DynamoDb,
    table_name: String,
    index_name: String,
    year: u64,
) -> Result<Option<Vec<Book>>, eyre::Error> {
    let result = client
        .query(QueryInput {
            table_name,
            index_name: Some(index_name),
            // key_condition_expression: Some("id = :id and begins_with(title, :t)".to_owned()),
            // expression_attribute_values: Some(attr_map! {
            //     ":id" => id,
            //     ":t" => book_title_begin
            // }),
            key_condition_expression: Some("book_year = :y".to_owned()),
            expression_attribute_values: Some(attr_map! {
                ":y" => year
            }),
            ..Default::default()
        })
        .await?
        .items;

    match result {
        Some(items_data) => {
            let mut result = Vec::new();
            result.reserve(items_data.len());

            for data in items_data.into_iter() {
                let book = Book::from_attrs(data).wrap_err("Book parsing")?;
                result.push(book);
            }

            Ok(Some(result))
        }
        None => Ok(None),
    }
}

////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[instrument(skip(client, book_key), fields(book_title))]
async fn update_book_year(
    client: &impl DynamoDb,
    table_name: String,
    book_key: Attributes,
    year: u64,
) -> Result<(), eyre::Error> {
    {
        let book_title = book_key
            .get("book_title")
            .and_then(|title| title.s.as_ref())
            .ok_or_else(|| eyre::eyre!("book_title value is missing"))?;
        tracing::span::Span::current().record("book_title", &book_title.as_str());
    }

    let res = client
        .update_item(UpdateItemInput {
            table_name,
            key: book_key,
            update_expression: Some("SET book_year = :y".to_owned()),
            expression_attribute_values: Some(attr_map! {
                ":y" => year
            }),
            ..Default::default()
        })
        .await?;

    info!(book_update_res = ?res, "Book year updated:");

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

    let table_name = "books";

    // Выводим таблички
    print_tables_list(&client).await?;

    // Создаем табличку
    create_table_if_needed(&client, table_name.to_owned()).await?;

    // Добавляем книжку
    let book = Book {
        title: "first_book_title".to_string(),
        book_year: 1999,
        authors: Some(vec![Author {
            name: "Book author title".to_string(),
        }]),
    };
    let new_book_key = book.key();
    insert_book(&client, table_name.to_owned(), book).await?;

    // Пытаемся получить эту книжку
    let received_book = get_book_with_title(&client, table_name.to_owned(), new_book_key.clone()).await?;
    info!(?received_book, "Book received:");

    // Делаем запрос на основании года
    let query_res =
        query_books_with_year_greater(&client, table_name.to_owned(), "year_index".to_owned(), 1999_u64).await?;
    info!(?query_res, "Book search received:");

    // Обновление года у конкретной книги
    update_book_year(&client, table_name.to_owned(), new_book_key.clone(), 2005).await?;

    Ok(())
}
