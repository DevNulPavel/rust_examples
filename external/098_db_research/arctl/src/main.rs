use anyhow::Context;
use arachne_client::{ArachneClient, SelectUser, User, UserId};
use clap::{Parser, Subcommand};
use std::net::SocketAddrV4;

/// Утилита управления сервером Arachne (arctl)
#[derive(Parser, Debug)]
#[command(author, version, long_about = None)]
struct Cli {
    /// Адрес TCP сервера Arachne
    #[arg(long, default_value = "127.0.0.1:50051", global = true)]
    host: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Управление платформами
    Platform {
        #[command(subcommand)]
        action: PlatformCommand,
    },

    /// Управление селекторами
    Selector {
        /// ID платформы
        #[arg(short, long)]
        platform: u8,

        #[command(subcommand)]
        action: SelectorCommand,
    },

    /// Управление пользователями
    User {
        /// ID платформы
        #[arg(short, long)]
        platform: u8,

        #[command(subcommand)]
        action: UserCommand,
    },

    /// Общая статистика сервера
    TotalUsersCount,
}

#[derive(Subcommand, Debug)]
enum PlatformCommand {
    /// Список всех платформ
    List,
    /// Создать новую платформу
    Create { id: u8 },
    /// Удалить платформу со всеми данными
    Delete { id: u8 },
    /// Количество пользователей на платформе
    Count { id: u8 },
}

#[derive(Subcommand, Debug)]
enum SelectorCommand {
    /// Создать или обновить селектор
    Create {
        /// ID селектора
        id: u8,
        /// запрос
        query: String,
        /// Время блокировки пользователя (сек)
        #[arg(short, long, default_value_t = 300)]
        lock_duration: u64,
    },
    /// Удалить селектор
    Delete { id: u8 },
    /// Узнать размер очереди готовых пользователей в селекторе
    Size { id: u8 },
    /// Узнать размер очереди ожидающих времени пользователей в селекторе
    PendingSize { id: u8 },
    /// Узнать размер очереди заблокированных пользователей в селекторе
    LockedSize { id: u8 },
    /// Вытащить одного пользователя для проверки (Select)
    Select { id: u8 },
}

#[derive(Subcommand, Debug)]
enum UserCommand {
    /// Найти пользователя по ID
    Search { id: String },
    /// Сбросить блокировку пользователя (прогнать по селекторам заново)
    Release { id: String },
    /// Удалить пользователя
    Delete { id: String },
    /// Добавить пользователя (JSON строка или путь к файлу: @payload.json)
    Add { payload: String },
    /// Обновить пользователя (JSON строка или путь к файлу: @payload.json)
    Update { payload: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let addr: SocketAddrV4 = cli.host.parse().context("Invalid host address")?;

    let client = ArachneClient::new(addr, 1)
        .await
        .context("Failed to connect to Arachne")?;

    match cli.command {
        Commands::TotalUsersCount => {
            let total = client.total_users_count().await?;
            println!("Total users across all platforms: {}", total);
        }

        Commands::Platform { action } => match action {
            PlatformCommand::List => {
                let list = client.platforms_list().await?;
                println!("Platforms ({}): {:?}", list.len(), list);
            }
            PlatformCommand::Create { id } => {
                let res = client.platform_create(id).await?;
                println!("Result: {:?}", res);
            }
            PlatformCommand::Delete { id } => {
                let res = client.platform_delete(id).await?;
                println!("Result: {:?}", res);
            }
            PlatformCommand::Count { id } => {
                let count = client.total_platformed_users_count(id).await?;
                println!("Platform {} total users: {}", id, count);
            }
        },

        Commands::Selector { platform, action } => match action {
            SelectorCommand::Create {
                id,
                query,
                lock_duration,
            } => {
                let res = client
                    .selector_create(platform, id, &query, lock_duration)
                    .await?;
                println!("Result: {:?}", res);
            }
            SelectorCommand::Delete { id } => {
                let res = client.selector_delete(platform, id).await?;
                println!("Result: {:?}", res);
            }
            SelectorCommand::Size { id } => {
                let size = client.selector_queue_size(platform, id).await?;
                println!("Selector {} queue size: {:?}", id, size);
            }
            SelectorCommand::Select { id } => {
                // Создаем легкого клиента/интерфейс для вызова select_user
                let res: SelectUser<serde_json::Value> = client.select_user(platform, id).await?;
                // Красиво печатаем в JSON формате
                println!("{}", serde_json::to_string_pretty(&res)?);
            }
            SelectorCommand::PendingSize { id } => {
                let size = client.selector_pending_size(platform, id).await?;
                println!("Selector {} pending size: {:?}", id, size);
            }
            SelectorCommand::LockedSize { id } => {
                let size = client.selector_locked_size(platform, id).await?;
                println!("Selector {} locked size: {:?}", id, size);
            }
        },

        Commands::User { platform, action } => match action {
            UserCommand::Search { id } => {
                let user_id = id
                    .parse::<i64>()
                    .map_or(UserId::Str(id.into()), UserId::Int);
                let res = client
                    .user_search::<serde_json::Value>(platform, user_id)
                    .await?;
                match res {
                    Some(user) => println!("{}", serde_json::to_string_pretty(&user)?),
                    None => println!("User not found."),
                }
            }
            UserCommand::Release { id } => {
                let user_id = id
                    .parse::<i64>()
                    .map_or(UserId::Str(id.into()), UserId::Int);
                let res = client.user_release(platform, user_id).await?;
                println!("Result: {:?}", res);
            }
            UserCommand::Add { payload } => {
                let user = parse_json_payload(&payload)?;
                let res = client.user_add(platform, user).await?;
                println!("Result: {:?}", res);
            }
            UserCommand::Update { payload } => {
                let user = parse_json_payload(&payload)?;
                let res = client.user_update(platform, user).await?;
                println!("Result: {:?}", res);
            }
            UserCommand::Delete { id } => {
                let user_id = id
                    .parse::<i64>()
                    .map_or(UserId::Str(id.into()), UserId::Int);
                let res = client.user_delete(platform, user_id).await?;
                println!("Result: {:?}", res);
            }
        },
    }

    Ok(())
}

fn parse_json_payload(input: &str) -> anyhow::Result<User<serde_json::Value>> {
    let json_str = if let Some(path) = input.strip_prefix('@') {
        std::fs::read_to_string(path).with_context(|| format!("Failed to read file: {}", path))?
    } else {
        input.to_string()
    };

    let user: User<serde_json::Value> =
        serde_json::from_str(&json_str).context("Failed to parse JSON into User structure")?;
    Ok(user)
}
