use turn::auth::*;
use turn::relay::relay_static::*;
use turn::server::{config::*, *};
use turn::Error;

use clap::{App, AppSettings, Arg};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::signal;
use tokio::time::Duration;
use util::vnet::net::*;

struct MyAuthHandler {
    cred_map: HashMap<String, Vec<u8>>,
}

impl MyAuthHandler {
    fn new(cred_map: HashMap<String, Vec<u8>>) -> Self {
        MyAuthHandler { cred_map }
    }
}

impl AuthHandler for MyAuthHandler {
    fn auth_handle(
        &self,
        username: &str,
        _realm: &str,
        _src_addr: SocketAddr,
    ) -> Result<Vec<u8>, Error> {
        if let Some(pw) = self.cred_map.get(username) {
            //log::debug!("username={}, password={:?}", username, pw);
            Ok(pw.to_vec())
        } else {
            Err(Error::ErrFakeErr)
        }
    }
}

// RUST_LOG=trace cargo run --color=always --package webrtc-turn --example turn_server_udp -- --public-ip 0.0.0.0 --users user=pass

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::init();

    let mut app = App::new("TURN Server UDP")
        .version("0.1.0")
        .author("Rain Liu <yliu@webrtc.rs>")
        .about("An example of TURN Server UDP")
        .setting(AppSettings::DeriveDisplayOrder)
        .setting(AppSettings::SubcommandsNegateReqs)
        .arg(
            Arg::with_name("FULLHELP")
                .help("Prints more detailed help information")
                .long("fullhelp"),
        )
        .arg(
            Arg::with_name("public-ip")
                .required_unless("FULLHELP")
                .takes_value(true)
                .long("public-ip")
                .help("IP Address that TURN can be contacted by."),
        )
        .arg(
            Arg::with_name("users")
                .required_unless("FULLHELP")
                .takes_value(true)
                .long("users")
                .help("List of username and password (e.g. \"user=pass,user=pass\")"),
        )
        .arg(
            Arg::with_name("realm")
                .default_value("webrtc.rs")
                .takes_value(true)
                .long("realm")
                .help("Realm (defaults to \"webrtc.rs\")"),
        )
        .arg(
            Arg::with_name("port")
                .takes_value(true)
                .default_value("3478")
                .long("port")
                .help("Listening port."),
        );

    let matches = app.clone().get_matches();

    // Печатаем помощь если надо
    if matches.is_present("FULLHELP") {
        app.print_long_help().unwrap();
        std::process::exit(0);
    }

    // Получаем параметры приложения
    let public_ip = matches.value_of("public-ip").unwrap();
    let port = matches.value_of("port").unwrap();
    let users = matches.value_of("users").unwrap();
    let realm = matches.value_of("realm").unwrap();

    ////////////////////////

    // Кешируем значения юзеров для быстрого обхода после
    // Если пароли сохранены - они должны быть сохранены в базе в виде хешей используя turn.GenerateAuthKey
    let creds: Vec<&str> = users.split(',').collect(); // Делим пользователей по запятой
    let mut cred_map = HashMap::new();
    for user in creds {
        // Делим пользователя на имя и пароль по символу '='
        let cred: Vec<&str> = user.splitn(2, '=').collect();
        // Генерим ключ на основе имени, области и пароля
        let key = generate_auth_key(cred[0], realm, cred[1]);
        // Сохарняем полученный ключ для имени юзера
        cred_map.insert(cred[0].to_owned(), key);
    }

    // Создаем UDP листенер для проброса в turn/pion
    let socket = UdpSocket::bind(format!("0.0.0.0:{}", port)).await?;
    let conn = Arc::new(socket);
    println!("listening {}...", conn.local_addr()?);

    // Создаем непосредственно сервер
    let server = Server::new(ServerConfig {
        conn_configs: vec![ConnConfig {
            conn,
            relay_addr_generator: Box::new(RelayAddressGeneratorStatic {
                relay_address: IpAddr::from_str(public_ip)?,
                address: "0.0.0.0".to_owned(),
                net: Arc::new(Net::new(None)),
            }),
        }],
        realm: realm.to_owned(),
        auth_handler: Arc::new(MyAuthHandler::new(cred_map)),
        channel_bind_timeout: Duration::from_secs(0),
    })
    .await?;

    println!("Waiting for Ctrl-C...");
    signal::ctrl_c().await.expect("failed to listen for event");
    println!("\nClosing connection now...");
    server.close().await?;

    Ok(())
}
