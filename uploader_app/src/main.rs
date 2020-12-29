mod env_parameters;
mod app_parameters;
mod uploaders;
mod result_senders;

use std::{
    path::{
        Path,
        PathBuf
    },
    error::{
        self
    }
};
use tokio::{
    runtime::{
        Builder
    }
};
use futures::{
    future::{
        Future,
        FutureExt,
        select_all,
        join_all
    }
};
use log::{
    info,
    debug,
    error
};
use slack_client_lib::{
    SlackClient,
    SlackError,
    SlackMessageTarget
};
use app_center_client::{
    AppCenterClient,
    AppCenterBuildGitInfo,
    AppCenterBuildUploadTask,
    AppCenterError
};
use self::{
    app_parameters::{
        AppParameters,
        SlackParams,
        AppCenterParams
    },
    env_parameters::{
        AppEnvValues,
        ResultSlackEnvironment,
        AppCenterEnvironment,
        GitEnvironment
    },
    uploaders::{
        upload_in_app_center,
        UploadResult
    },
    result_senders::{
        ResultSender,
        SlackResultSender,
        TerminalSender
    }
};

async fn wait_results<W, S>(mut active_workers: Vec<W>, 
                            mut result_senders: Vec<Box<S>>)
where 
    W: Future<Output=UploadResult> + Unpin,
    S: ResultSender + ?Sized {

    // Смотрим на завершающиеся воркеры
    while active_workers.len() > 0 {
        // Выбираем успешную фьючу, получаем оставшиеся
        let (res, _, left_workers) = select_all(active_workers).await;
        active_workers = left_workers;

        // Обрабатываем результат
        match res {
            Ok(res) => {
                /*let mut futures = Vec::new();
                for mut sender in result_senders{
                    let fut = sender.send_result(&res);
                    futures.push(fut);
                }*/

                // Пишем во все получатели асинхронно
                let futures_iter = result_senders
                    .iter_mut()
                    .map(|sender|{
                        sender.send_result(&res)
                    });
                join_all(futures_iter).await;
            },
            Err(err) => {
                // Пишем во все получатели асинхронно
                let futures_iter = result_senders
                    .iter_mut()
                    .map(|sender|{
                        sender.send_error(err.as_ref())
                    });
                join_all(futures_iter).await;

                error!("Uploading task failed: {}", err);
            }
        }
    }
}

async fn async_main() {
    // Параметры приложения
    let app_parameters = AppParameters::parse(Some(||{
        AppEnvValues::get_possible_env_variables()    
            .into_iter()
            .fold(String::from("ENVIRONMENT VARIABLES:\n"), |mut prev, var|{
                prev.push_str("    - ");
                prev.push_str(var);
                prev.push_str("\n");
                prev
            })
    }));
    
    // Получаем параметры окружения
    let env_params = AppEnvValues::parse();

    // Общий клиент для запросов
    let http_client = reqwest::Client::new();

    // Вектор с активными футурами выгрузки
    let mut active_workers = Vec::new();

    // Создаем задачу выгрузки в AppCenter
    match (env_params.app_center, app_parameters.app_center) {
        (Some(app_center_env_params), Some(app_center_app_params)) => {
            let fut = upload_in_app_center(http_client.clone(), 
                                           app_center_env_params, 
                                           app_center_app_params,
                                           env_params.git).boxed();
            active_workers.push(fut);
        },
        _ => {}
    }

    // Получаетели результатов выгрузки
    let result_senders = {
        let mut result_senders: Vec<Box<dyn ResultSender>> = Vec::new();

        // Создаем клиента для слака если надо отправлять результаты в слак
        if let Some(slack_params) = env_params.result_slack{
            
        }

        // Результат в терминал
        result_senders.push(Box::new(TerminalSender{}));

        result_senders
    };

    wait_results(active_workers, result_senders).await;
}

fn setup_logs(){
    // Активируем логирование и настраиваем уровни вывода
    // https://rust-lang-nursery.github.io/rust-cookbook/development_tools/debugging/config_log.html
    //#[cfg(debug_assertions)]
    {
        if !std::env::var("RUST_LOG").is_ok() {
            std::env::set_var("RUST_LOG", "uploader_app=trace");
        }
    }
    env_logger::init();
}

fn main() {
    // Активируем логирование и настраиваем уровни вывода
    setup_logs();

    // Запускаем асинхронный рантайм
    let mut runtime = Builder::default()
        .enable_io()
        .basic_scheduler()
        //.threaded_scheduler()
        //.core_threads(1)
        //.max_threads(2)
        .build()
        .expect("Tokio runtime create failed");

    runtime.block_on(async_main());

    // Dump the report to disk
    #[cfg(feature = "flame_it")]
    flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
}
