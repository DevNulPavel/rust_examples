
#[derive(Debug)]
pub struct FacebookEnvParams{
    pub client_id: String,
    pub client_secret: String
}

impl FacebookEnvParams {
    pub fn get_from_env() -> FacebookEnvParams {
        let client_id = std::env::var("FACEBOOK_CLIENT_ID")
            .expect("FACEBOOK_CLIENT_ID environment variable is missing");
        let client_secret = std::env::var("FACEBOOK_CLIENT_SECRET")
            .expect("FACEBOOK_CLIENT_SECRET environment variable is missing");

        FacebookEnvParams{
            client_id,
            client_secret
        }
    }
}