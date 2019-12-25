extern crate test3_module; // Таким образом нужно использовать библиотеку

use test3_module::client;   // Указываем что именно мы должны использовать из библиотеки
use test3_module::*;   // Таким образом можно импортировать все из библиотеки

fn main() {
    client::connect();
    test3_module::server::connect(); // Использование полного пути из библиотеки
    server::connect();
}