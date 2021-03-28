pub struct GoogleEnvParams{
    pub client_id: String,
    pub client_secret: String
}

impl GoogleEnvParams {
    pub fn get_from_env() -> GoogleEnvParams {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .expect("GOOGLE_CLIENT_ID environment variable is missing");
        let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .expect("GOOGLE_CLIENT_SECRET environment variable is missing");

        GoogleEnvParams{
            client_id,
            client_secret
        }
    }
}