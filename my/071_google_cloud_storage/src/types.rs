use hyper::{
    body::Body as BodyStruct,
    client::connect::{dns::GaiResolver, HttpConnector},
    Client,
};
use hyper_rustls::HttpsConnector;

pub type HttpClient = Client<HttpsConnector<HttpConnector<GaiResolver>>, BodyStruct>;
