use url::{
    Url
};
// #[derive(Debug)]
// pub struct AppParametersHttps{
//     pub port: u32,
//     pub cert_file_path: PathBuf
// }

#[derive(Debug)]
pub struct AppParameters{
    pub app_base_url: Url,
    pub game_url: Url,
    pub http_port: u32,
    //pub https_config: Option<AppParametersHttps>
}

impl AppParameters {
    pub fn get_from_env() -> AppParameters {
        let app_base_url = std::env::var("APP_URL_ADDR")
            .ok()
            .and_then(|url|{
                Url::parse(&url).ok()
            })
            .expect("APP_URL_ADDR parsing failed");
        let game_url = std::env::var("GAME_URL_ADDR")
            .ok()
            .and_then(|url|{
                Url::parse(&url).ok()
            })
            .expect("GAME_URL parsing failed");
        let http_port = std::env::var("HTTP_PORT")
            .expect("HTTP_PORT environment variable is missing")
            .parse::<u32>()
            .expect("HTTP_PORT parsing failed");

        AppParameters{
            app_base_url,
            game_url,
            http_port
        }
    }
}