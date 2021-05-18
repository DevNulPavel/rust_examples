use std::{
    sync::{
        Arc, 
        RwLock
    },
    collections::{
        HashMap
    }
};
use colored::{
    Colorize
};
use log::{
    debug
};
use super::{
    StreamId
};

#[derive(Debug, Clone)]
pub struct Log {
    method: String,
    path: String,
}

lazy_static::lazy_static! {
    pub static ref LOGS: Arc<RwLock<HashMap<StreamId, Log>>> = Arc::new(RwLock::new(HashMap::new()));
}

pub fn connect_failed() {
    eprintln!("{}", "CONNECTION REFUSED".red())
}

/// Логируем входящее сообщение?
pub fn log_incoming(stream_id: StreamId, data: Vec<u8>) {
    // Если уже есть такие данные, то не надо еще раз
    if LOGS.read().unwrap().contains_key(&stream_id) {
        return;
    }

    // Заголовки
    let mut headers = [httparse::EMPTY_HEADER; 64];
    
    // Запрос
    let mut req = httparse::Request::new(&mut headers);

    // Метод и путь запроса
    let (method, path) = match req.parse(&data) {
        Ok(_status) => {
            match (req.method, req.path) {
                (Some(m), Some(p)) => (m,p),
                _ => {
                    debug!("Incomplete request, skipping.");
                    return
                }
            }
        },
        Err(e) => {
            debug!("Invalid request: {:?}", e);
            return
        }
    };

    // Записываем лог
    LOGS
        .write()
        .unwrap()
        .insert(stream_id, Log { 
            method: method.to_string(), 
            path: path.to_string() 
        });
}

/// Логируем исходящее сообщение
pub fn log_outgoing(stream_id: StreamId, data: Vec<u8>) {
    // Получаем блокировку
    let mut logs = LOGS.write().unwrap();

    // Получаем ссылку на лог
    let log: &Log = match logs.get(&stream_id) {
        Some(l) => {
            l
        },
        None => {
            debug!("no log line for response");
            return
        }
    };

    let mut headers = [httparse::EMPTY_HEADER; 30];
    let mut resp = httparse::Response::new(&mut headers);

    let _ = resp
        .parse(&data)
        .map_err(|e| debug!("error parsing response: {:?}", e));

    let out = match resp.code {
        Some(code @ 200..=299) => {
            format!("{}", code).green()
        }
        Some(code) => {
            format!("{}", code).red()
        }
        _ => {
            "???".red()
        }
    };

    eprint!("{}", out);

    eprintln!("\t\t{}\t{}", log.method.to_uppercase().yellow(), log.path.blue());
    logs.remove(&stream_id);
}
