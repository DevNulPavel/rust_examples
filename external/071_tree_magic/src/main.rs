//! `tmagic`: Command line client for tree_magic
//!
//! # Features
//! - Find MIME of a file
//! - Match file against a set of MIMEs (significantly faster)
//! - Search a folder recursively
//!
//! # Usage
//!
//! ```
//! tmagic [FLAGS] [OPTIONS] <file>...
//!
//! FLAGS:
//!     -h, --help             Prints help information
//!     -r, --recursive        Search directories recursively
//!         --ugly             Print results as they come in, at expense of tab alignment
//!     -V, --version          Prints version information
//!
//! OPTIONS:
//!     -m, --match=<match>    Print only files that match given MIMEs, seperated by commas
//!
//! ARGS:
//!     <file>...              List of files or folders to check. Wildcards supported. 

use tabwriter::TabWriter;
use std::io::prelude::*;
use std::sync::mpsc;
use std::path::PathBuf;
use walkdir::{WalkDir};
use scoped_threadpool::Pool;
use clap::{Arg, App};

macro_rules! convmime {
    ($x:expr) => {$x.to_string()}
}

fn main() {
    // Парсим аргументы командной стоки
    let args = App::new("TreeMagic")
        .version("0.2.0")
        .about("Determines the MIME type of a file by traversing a filetype tree.")
        // Список файликов для определения типа
        .arg(Arg::with_name("file")
            .required(true)
            .index(1)
            .multiple(true)
            .help("List of files or folders to check. Wildcards supported.")
        )
        // Рекурсивно обходим директории до 1го уровня
        .arg(Arg::with_name("recursive")
            .short("r")
            .long("recursive")
            .help("Search directories recursively")
        )
        // Только файлики, соответствующие определенному MIME
        .arg(Arg::with_name("match")
            .short("m")
            .long("match")
            .use_delimiter(true)
            .takes_value(true)
            .require_equals(true)
            .help("Print only files that match given MIMEs, seperated by commas")
        )
        // Выводим результаты как есть, без красивого вывода
        .arg(Arg::with_name("ugly")
            .long("ugly")
            .help("Print results as they come in, at expense of tab alignment")
        )
        .get_matches();
        
    // Конвертируем список файликов в PathBuf
    let mut files: Vec<PathBuf> = args.values_of("file")
        .unwrap()
        .map(|x| PathBuf::from(x))
        .collect();
    // Некрасивый вывод
    let is_ugly = args.is_present("ugly");
    // Рекурсивно
    let is_recursive = args.is_present("recursive");
    // Соответствующие MIME маски
    let check_against: Vec<String> = match args.values_of("match") {
        Some(y) => { y.map(|x| x.to_string()).collect() }
        None => Vec::<String>::new()
    };
    
    // Создаем обработчик расивого вывода в виде табов
    let mut tw = TabWriter::new(vec![]);

    // Канал для результатов
    let (tx, rx) = mpsc::channel();
    
    // Обходим список файликов и дополняем с первогоу уровня рекурсии
    if is_recursive {
        for dir in files.clone() {
            let entries = WalkDir::new(dir).into_iter().filter_map(|e| e.ok());
            for entry in entries {
                files.push(PathBuf::from(entry.path()));
            }
        }
    }
    let files = files;
    
    // Создаем пул потоков
    let mut pool = Pool::new(num_cpus::get() as u32);

    // Надо ли делать проверку по маске?
    if check_against.is_empty(){
        // Создаем скоуп потоков, на котором мы блокируемся
        pool.scoped(|scope| {
            let process_files = |files|{
                // Обходим список файликов
                for file in files {
                    let tx = tx.clone();
                    scope.execute(move || {
                        let mime_result = tree_magic::from_filepath(file.as_path());
                        let result = format!("{:?}:\t{:?}", file, mime_result);
                        if is_ugly {
                            println!("{}", result);
                        } else {
                            tx.send(result + "\n").unwrap_or_default();
                        }
                    });
                }
            };
            
            // Чтобы не городить очень много лямбд
            // Мы можем разделить на чанки
            if file.len()/pool.thread_count() > 100 {
                let chunks: Vec<_> = files.chunks(pool.thread_count()).collect();
                chunks.into_iter().for_each(process_files);
            }else{
                process_files(files);
            }
        });
    // Acquire results for check against list of MIMES
    } else {
        pool.scoped(|scope| {
            for file in files {
                //let file = file.as_str();
                //let file = Path::new(file);
                let tx = tx.clone();
                let check_against = check_against.clone();
                
                scope.execute(move || {
                    let mut result: Option<String> = None;
                
                    for mime in check_against {
                        let out = tree_magic::match_filepath(mime.as_str(), file.as_path());
                        if out {
                            result = Some(mime);
                            break;
                        }
                    }
                    
                    if result.is_none() { return; }
                    
                    let result = result.unwrap();
                    let result = format!("{:?}:\t{:?}", file, result);
                    if is_ugly {
                        println!("{}", result);
                    } else {
                        tx.send(result + "\n").unwrap_or_default();
                    }
                });
            }
        });
    }
    drop(tx);
    
    // Pretty-print results
    if !is_ugly {
        let mut list: Vec<_> = rx.iter().collect();
        list.sort();
        list.dedup();
        for x in list {
            write!(&mut tw, "{}", x).unwrap();
        }
        
        tw.flush().unwrap();
        let out = String::from_utf8(tw.into_inner().unwrap()).unwrap_or("".to_string());
        println!("{}", out);
    }
    
}
