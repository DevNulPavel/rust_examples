mod seeder;

use anyhow::Result;
use clap::Parser;
use iggy::args::{Args, ArgsOptional};
use iggy::client::{Client, UserClient};
use iggy::client_provider;
use iggy::client_provider::ClientProviderConfig;
use iggy::clients::client::IggyClient;
use iggy::utils::crypto::{Aes256GcmEncryptor, Encryptor};
use std::error::Error;
use std::sync::Arc;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct DataSeederArgs {
    #[clap(flatten)]
    pub(crate) iggy: ArgsOptional,

    #[arg(long, default_value = "iggy")]
    pub username: String,

    #[arg(long, default_value = "iggy")]
    pub password: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = DataSeederArgs::parse();
    let iggy_args = Args::from(vec![args.iggy.clone()]);

    Registry::default()
        .with(tracing_subscriber::fmt::layer())
        .with(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO")))
        .init();
    let encryptor: Option<Arc<dyn Encryptor>> = match iggy_args.encryption_key.is_empty() {
        true => None,
        false => Some(Arc::new(
            Aes256GcmEncryptor::from_base64_key(&iggy_args.encryption_key).unwrap(),
        )),
    };
    info!("Selected transport: {}", iggy_args.transport);
    let username = args.username.clone();
    let password = args.password.clone();
    let client_provider_config = Arc::new(ClientProviderConfig::from_args(iggy_args)?);
    let client = client_provider::get_raw_client(client_provider_config, false).await?;
    let client = IggyClient::create(client, None, encryptor);
    client.connect().await?;
    client.login_user(&username, &password).await.unwrap();
    info!("Data seeder has started...");
    seeder::seed(&client).await.unwrap();
    info!("Data seeder has finished.");
    Ok(())
}
