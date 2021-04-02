use clap::{
    Arg, 
    App
};
use url::{
    Url
};

#[derive(Debug)]
pub struct AppParameters{
    pub game_url: Url
}

impl AppParameters {
    pub fn parse() -> AppParameters{
        let matches = App::new("OAuth server")
            .version("1.0")
            .author("Pavel Ershov")
            .arg(Arg::with_name("game_url")
                    .short("u")
                    .long("game_url")
                    .value_name("GAME_URL")
                    .help("Game url for redirect")
                    .required(true)
                    .takes_value(true))                    
            .get_matches();

        // Получаем входные значения в нужном формате
        let game_url = matches
            .value_of("game_url")
            .and_then(|entry|{
                Url::parse(entry).ok()
            })
            .expect("Invalid game redirect url");

        AppParameters{
            game_url
        }
    }
}