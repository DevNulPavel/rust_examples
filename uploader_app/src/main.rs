mod env_parameters;
mod app_parameters;
mod uploaders;
mod result_senders;

use std::{
    pin::{
        Pin
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
use self::{
    app_parameters::{
        AppParameters
    },
    env_parameters::{
        AppEnvValues,
        ResultSlackEnvironment
    },
    uploaders::{
        upload_in_app_center,
        upload_in_google_drive,
        upload_in_google_play,
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

fn build_uploaders(http_client: reqwest::Client, 
                   env_params: AppEnvValues, 
                   app_parameters: AppParameters) -> (Option<ResultSlackEnvironment>, Vec<Pin<Box<dyn Future<Output=UploadResult> + Send>>>) {

    let mut active_workers = Vec::new();

    // Создаем задачу выгрузки в AppCenter
    match (env_params.app_center, app_parameters.app_center) {
        (Some(app_center_env_params), Some(app_center_app_params)) => {
            info!("App center uploading task created");
            let fut = upload_in_app_center(http_client.clone(), 
                                           app_center_env_params, 
                                           app_center_app_params,
                                           env_params.git).boxed();
            active_workers.push(fut);
        },
        _ => {}
    }

    // Создаем задачу выгрузки в Google drive
    match (env_params.google_drive, app_parameters.goolge_drive) {
        (Some(env_params), Some(app_params)) => {
            info!("Google drive uploading task created");
            let fut = upload_in_google_drive(http_client.clone(),
                                             env_params, 
                                             app_params).boxed();
            active_workers.push(fut);
        },
        _ => {}
    }

    // Создаем задачу выгрузки в Google Play
    match (env_params.google_play, app_parameters.goolge_play) {
        (Some(env_params), Some(app_params)) => {
            info!("Google play uploading task created");
            let fut = upload_in_google_play(http_client,
                                            env_params, 
                                            app_params).boxed();
            active_workers.push(fut);
        },
        _ => {}
    }

    (env_params.result_slack, active_workers)
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
    
    debug!("App params: {:#?}", app_parameters);

    // Получаем параметры окружения
    let env_params = AppEnvValues::parse();

    debug!("Env params: {:#?}", env_params);

    // Общий клиент для запросов
    let http_client = reqwest::Client::new();

    // Вектор с активными футурами выгрузки
    let (result_slack, active_workers) = build_uploaders(http_client.clone(), env_params, app_parameters);

    // Получаетели результатов выгрузки
    let result_senders = {
        let mut result_senders: Vec<Box<dyn ResultSender>> = Vec::new();

        // Создаем клиента для слака если надо отправлять результаты в слак
        if let Some(slack_params) = result_slack{
            let slack_sender = SlackResultSender::new(http_client, slack_params);
            result_senders.push(Box::new(slack_sender));    
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
        .enable_all()
        // .basic_scheduler()
        .threaded_scheduler()
        //.core_threads(1)
        //.max_threads(2)
        .build()
        .expect("Tokio runtime create failed");

    runtime.block_on(async_main());

    // Dump the report to disk
    #[cfg(feature = "flame_it")]
    flame::dump_html(&mut std::fs::File::create("flame-graph.html").unwrap()).unwrap();
}
