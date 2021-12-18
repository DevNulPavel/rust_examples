use structopt::StructOpt;

/// App parameters
#[derive(StructOpt, Debug)]
#[structopt(name = "basic")]
pub struct AppArguments {
    /// Request token for uploading
    #[structopt(long, env = "MONGODB_CONNECTION_ADDR")]
    pub mongodb_connection_addr: String,
}
