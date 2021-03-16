extern crate ansi_term;
#[macro_use]
extern crate clap;
extern crate hyper;
extern crate libzou;
extern crate num_cpus;

#[macro_use]
mod logs;

use clap::{
    App, 
    Arg
};
use std::{
    fs::{
        File, 
        remove_file
    },
    error::{
        Error
    },
    path::{
        Path
    },
    process::{
        exit
    }
};
use libzou::{
    cargo_helper::{
        get_remote_server_informations
    },
    download::{
        download_chunks
    },
    filesize::{
        StringFileSize
    },
    protocol::{
        get_protocol, 
        Protocol
    },
    util::{
        prompt_user
    },
    write::{
        OutputFileWriter
    }
};

fn main() {
    // Parse arguments
    let argparse = App::new("Zou")
        .about("Zou, a simple and fast download accelerator, written in Rust.")
        .version(crate_version!())
        .arg(Arg::with_name("threads")
                 .long("threads")
                 .short("t")
                 .takes_value(true)
                 .help("Threads which can use to download"))
        .arg(Arg::with_name("debug")
                 .long("debug")
                 .short("d")
                 .help("Active the debug mode"))
        .arg(Arg::with_name("force")
                 .long("force")
                 .help("Assume Yes to all queries and do not prompt"))
        .arg(Arg::with_name("mirrors")
                 .long("mirrors")
                 .short("m")
                 .multiple(true)
                 .takes_value(true)
                 .help("Download using a list of mirrors - the list of mirrors is used WITH the original URL"))
        .arg(Arg::with_name("output")
                .long("output")
                .short("o")
                .takes_value(true)
                .help("Specify the local output"))
        .arg(Arg::with_name("ssl_support")
                .long("ssl_support")
                .short("s")
                .help("Switch to an SSL client"))
        .arg(Arg::with_name("url")
            .index(1)
            //.multiple(true)
            .required(true))
        .get_matches();

    // Get informations from arguments

    // Создаем Path для нашего URL
    let url = Path::new(argparse.value_of("url").unwrap());
    let url_str = url.to_str().unwrap();

    // Получаем имя файлика из пути
    let filename = url.file_name().unwrap().to_str().unwrap();

    // Конвертим значение параметра в usize
    // Если не 0, тогда используем это количество
    let mut threads: usize = value_t!(argparse, "threads", usize)
        .and_then(|v| {
            if v != 0 {
                Ok(v)
            } else {
                Err(clap::Error::with_description("Cannot download a file using 0 thread",
                                                  clap::ErrorKind::InvalidValue,
                ))
            }
        })
        .unwrap_or(num_cpus::get_physical());

    // Есть ли флаг отладки?
    if argparse.is_present("debug") {
        info!(&format!("zou V{}", crate_version!()));
        info!(&format!("downloading {}, using {} threads", filename, threads));
    }

    // Создаем объект пути
    let local_path = Path::new(argparse.value_of("output").unwrap_or(&filename));

    // Есть ли уже файлик?
    if local_path.exists() {
        // Если локальный путь - это папка
        if local_path.is_dir() {
            epanic!("The local path to store the remote content is already exists, \
                        and is a directory!");
        }

        // Если не прокинут флаг принудительной перезаписи, тогда спрашиваем перезаписать или нет
        if !argparse.is_present("force") {
            let user_input = prompt_user("The path to store the file already exists! \
                                          Do you want to override it? [y/N]");
            if !(user_input == "y" || user_input == "Y") {
                exit(0);
            }
        } else {
            warning!(
                "The path to store the file already exists! \
                                 It is going to be overriden."
            );
        }
    }

    // Получаем протокол, HTTPS/HTTP
    let ssl_support = match get_protocol(url.to_str().unwrap()) {
        Some(protocol) => {
            match protocol {
                // Если протокол - http, тогда возвращаем пользовательский выбор HTTPS клиента
                Protocol::HTTP => argparse.is_present("ssl_support"),
                // Форсированно используем HTTPS
                Protocol::HTTPS => true,
            }
        }
        None => {
            epanic!("Unknown protocol!")
        },
    };

    // Получаем информацию из удаленного сервера, для того, чтобы выполнить загрузку правильнее
    let remote_server_informations = match get_remote_server_informations(url_str, ssl_support) {
        Ok(mut informations) => {
            // Проверяем, была ли запрошена загрузка в один поток
            // На основании этого выставляем возможность частичной загрузки данных
            informations.accept_partialcontent = !(threads == 1);
            // Возвращаем результат
            informations
        }
        Err(err) => {
            error!(&format!("Getting remote server informations: {}",
                                err.description()));
            exit(1);
        }
    };

    // Длина контента
    info!(&format!("Remote content length: {}", StringFileSize::from(remote_server_informations.file.content_length)));

    // Создаем файлик
    let local_file = File::create(local_path)
        .expect("[ERROR] Cannot create a file !");

    // Выставляем размер файлика
    local_file
        .set_len(remote_server_informations.file.content_length)
        .expect("Cannot extend local file !");
    let out_file = OutputFileWriter::new(local_file);

    // Если сервер не принимает PartialContent статус, загружаем удаленный файлик лишь в одном потоке
    if !remote_server_informations.accept_partialcontent {
        warning!(
            "The remote server does not accept PartialContent status! \
                             Downloading the remote file using one thread."
        );
        threads = 1;
    }

    // Выполняем фактическую загрузку файлика
    let res = download_chunks(remote_server_informations,
                              out_file, 
                              threads as u64,
                              ssl_support);
    if res {
        ok!(&format!("Your download is available in {}",
                     local_path.to_str().unwrap()));
    } else {
        // Если файлик не в порядке, тогда удаляем его из файловой системы
        error!("Download failed! An error occured - erasing file... ");
        if remove_file(local_path).is_err() {
            error!("Cannot delete downloaded file!");
        }
    }
}
