use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// MongoDB connection address
    #[structopt(long, env = "MONGODB_CONNECTION_ADDR")]
    pub mongodb_connection_addr: String,

    /// Database name for open
    #[structopt(long, env = "MONGODB_DATABASE_NAME")]
    pub mongodb_database_name: String,

    /// Collection name for open
    #[structopt(long, env = "MONGODB_COLLECTION_NAME")]
    pub mongodb_collection_name: String,
}
