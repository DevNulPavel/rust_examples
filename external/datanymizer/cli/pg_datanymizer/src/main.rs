mod options;

use anyhow::{
    Result
};
use datanymizer_dumper::{
    postgres::{
        dumper::{
            PgDumper
        }
    },
    Dumper
};
use datanymizer_engine::{
    Engine, 
    Settings
};
use options::{
    Options
};
use postgres::{
    Client, 
    NoTls
};
use structopt::{
    StructOpt
};

fn main() -> Result<()> {
    // Параметры
    let cfg = Options::from_args();

    // Парсим урл из параметров
    let url = cfg.database_url()?;

    let conf = cfg
        .clone()
        .config
        .unwrap_or_else(|| {
            "./config.yml".to_string()
        });
    let s = Settings::new(conf, url.clone())?;

    let mut client = Client::connect(&url, NoTls)?;
    let mut dumper = PgDumper::new(Engine::new(s), cfg.pg_dump_location, cfg.file)?;
    dumper.dump(&mut client)
}
