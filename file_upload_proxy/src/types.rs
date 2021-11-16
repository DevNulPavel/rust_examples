use crate::{app_arguments::AppArguments, auth_token_provider::AuthTokenProvider};
use hyper::{
    body::Body as BodyStruct,
    client::connect::{dns::GaiResolver, HttpConnector},
    Client,
};
use hyper_rustls::HttpsConnector;

//////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub type HttpClient = Client<HttpsConnector<HttpConnector<GaiResolver>>, BodyStruct>;

pub struct App {
    pub app_arguments: AppArguments,
    pub http_client: HttpClient,
    pub token_provider: AuthTokenProvider,
}
