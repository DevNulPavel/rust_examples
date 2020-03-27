extern crate actix;
extern crate actix_web;
extern crate chrono;
extern crate crypto;
extern crate futures;
extern crate rand;
extern crate url;
extern crate url_serde;
extern crate uuid;

#[macro_use]
extern crate serde_derive;
extern crate serde;
extern crate serde_json;



mod core;
mod web;

pub use web::Server;
