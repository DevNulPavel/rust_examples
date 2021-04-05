use std::{
    path::{
        PathBuf
    }
};
use clap::{
    Arg, 
    App
};
use url::{
    Url
};

#[derive(Debug)]
pub struct AppParametersHttps{
    pub port: u32,
    pub cert_file_path: PathBuf
}

#[derive(Debug)]
pub struct AppParameters{
    pub game_url: Url,
    pub http_port: u32,
    //pub https_config: Option<AppParametersHttps>
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
            .arg(Arg::with_name("http_port")
                    .short("p")
                    .long("http_port")
                    .value_name("HTTP_PORT")
                    .help("HTTP port value")
                    .takes_value(true)
                    .required(true))
            /*.arg(Arg::with_name("http_port")
                    .short("p")
                    .long("http_port")
                    .value_name("HTTP_PORT")
                    .help("HTTP port value")
                    .takes_value(true)
                    .required_unless("https_port"))
            .arg(Arg::with_name("https_port")
                    .short("s")
                    .long("https_port")
                    .value_name("HTTPS_PORT")
                    .help("HTTPs port value")
                    .takes_value(true)
                    .requires("https_cert_file")
                    .required_unless("http_port"))
            .arg(Arg::with_name("https_cert_file")
                    .short("c")
                    .long("https_cert_file")
                    .value_name("HTTPS_CERT_FILE")
                    .help("HTTPs certificate value")
                    .takes_value(true))*/ 
            .get_matches();

        // Получаем входные значения в нужном формате
        let game_url = matches
            .value_of("game_url")
            .and_then(|entry|{
                Url::parse(entry).ok()
            })
            .expect("Invalid game redirect url");

        let http_port = matches
            .value_of("http_port")
            .and_then(|entry|{
                entry.parse::<u32>().ok()
            })
            .expect("HTTP port parsing failed");

        /*let https_port = matches
                .value_of("https_port")
                .and_then(|entry|{
                    entry.parse::<u32>().ok()
                });

        let https_cert_file = matches
                .value_of("https_cert_file")
                .map(|entry|{
                    PathBuf::from(entry)
                });

        let https_config = match (https_port, https_cert_file) {
            (Some(port), Some(cert)) => {
                assert!(cert.exists(), "Https certificate file does not exist");
                Some(AppParametersHttps{
                    port,
                    cert_file_path: cert
                })
            },
            _ => {
                None
            }
        };*/

        AppParameters{
            game_url,
            http_port,
            //https_config
        }
    }
}