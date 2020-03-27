use std::error::Error;
use std::{env, process};
use test37_blockchain::Server;


fn main() -> Result<(), Box<dyn Error>> {
    // Инициализация логирования
    pretty_env_logger::init();

    // Получаем порт сервера
    let port = env::var("PORT")
        .unwrap_or(String::from("8088"))
        .parse()?;

    // Новый сервер
    let server = Server::new(port);

    // Выходим
    let exit_code = server.run()?;

    if exit_code == 0 {
        Ok(())
    } else {
        process::exit(exit_code)
    }
}
