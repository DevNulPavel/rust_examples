use std::process::Output;
use std::path::Path;
use std::sync::Arc;
use tokio::process::{ Command };
use tokio::sync::mpsc::unbounded_channel;
use crate::types::{StringResult, EmptyResult, PathReceiver, ProcessCommand, PathSender};

/// Расчитываем md5 c помощью подпроцесса
async fn get_md5(path: &Path) -> StringResult {
    let path_str = match path.to_str(){
        Some(path_str) => {
            path_str
        },
        None => {
            return Err("Empty path".into());
        }
    };

    let child = Command::new("md5")
        .arg(path_str)
        .output();

    let out: Output = child.await?;

    println!("the command exited with: {:?}", out);

    if out.status.success() {
        let text = String::from_utf8(out.stdout)?;
        println!("Text return: {}", text);
        Ok(text)
    }else{
        let text = String::from_utf8(out.stderr)?;
        println!("Error return: {}", text);
        Err(text.into())
    }
}

/// Обработка файлов из ресивера
async fn process_files(mut receiver: PathReceiver)-> EmptyResult {
    const MAX_COUNT: usize = 8;

    // Ограничение на 8 активных задач
    let semaphore = Arc::from(tokio::sync::Semaphore::new(MAX_COUNT));

    loop {
        let (received_path, response_ch) = match receiver.recv().await{
            // Получаем комманду
            Some(comand) => {
                match comand {
                    // Обработка данных
                    ProcessCommand::Process(data) => { 
                        data
                    },
                    ProcessCommand::Stop => {
                        break;
                    }
                }
            },
            None => {
                break;
            }
        };

        // Клон семафора
        let semaphore_clone = semaphore.clone();
        tokio::spawn(async move {
            // Берем блокировку
            let acquire = semaphore_clone.acquire().await;

            // Получаем MD5 асинхронно
            if let Ok(md5_res) = get_md5(&received_path).await {
                println!("Send to channel: {}", md5_res);
                // Расчитали файлик - удаляем
                let _ = tokio::fs::remove_file(&received_path).await;
                                
                let nanos = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .subsec_nanos();
                let time: u64 = (1000 + nanos % 4000) as u64;
                tokio::time::delay_for(std::time::Duration::from_millis(time)).await;
                
                // Отправляем исходный путь + md5
                let _ = response_ch.send((received_path, md5_res)); // TODO: ???
                println!("Send to channel success");
            }
            drop(acquire);
        });
    }

    // Перехватываем блокировку уже в текущем потоке, чтобы дождаться завершения всех потоков
    let waits: Vec<_> = (0..MAX_COUNT)
        .map(|_|{
            semaphore.acquire()
        })
        .collect();

    // Ждем завершения всех футур
    futures::future::join_all(waits).await;

    println!("Processing exit");
    Ok(())
}

pub struct Processing{
    handle: tokio::task::JoinHandle<EmptyResult>,
    sender: PathSender
}

impl Processing{
    pub fn new() -> Processing {
        let (processing_sender, input_receiver) = unbounded_channel::<ProcessCommand>();
        
        let process_file_future = tokio::spawn(process_files(input_receiver));
    
        Processing{
            handle: process_file_future, 
            sender: processing_sender
        }
    }

    pub async fn gracefull_finish_and_wait(self){
        // Завершаем обработку и ждем завершения
        self.sender.send(ProcessCommand::Stop).unwrap();
        if let Err(e) = self.handle.await{
            eprintln!("Process file error: {:?}", e);
        }
    }

    pub fn get_sender_clone(&self) -> PathSender{
        self.sender.clone()
    }
}