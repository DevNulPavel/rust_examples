
use log::{
    info
};
use reqwest::{
    Client
};
use super::{
    JenkinsClient,
    JenkinsRequestBuilder
};

#[actix_rt::test]
async fn test_jenkins_jobs() {
    std::env::set_var("RUST_LOG", "actix_server=info,actix_web=info,slack_command_handler=trace");
    env_logger::init();

    // Jenkins user
    let jenkins_user = std::env::var("JENKINS_USER")
        .expect("JENKINS_USER environment variable is missing");

    // Jenkins api token
    let jenkins_api_token = std::env::var("JENKINS_API_TOKEN")
        .expect("JENKINS_API_TOKEN environment variable is missing");

    // Общий менеджер запросов с пулом соединений
    // TODO: Configure
    let request_client = Client::new();

    let request_builder = JenkinsRequestBuilder::new(request_client, jenkins_user, jenkins_api_token);

    let client = JenkinsClient::new(request_builder);

    let jobs = client
        .request_jenkins_targets_list()
        .await
        .expect("Jobs list failed");

    let found_job = jobs
        .iter()
        .find(|job|{
            job.get_info().name == "slack_bot_test_target"
        })
        .expect("Required job is not found");

    // TODO: ???
    let _found_parameters = found_job
        .request_jenkins_target_info()
        .await
        .expect("Job parameter request error");

    let parameters = serde_json::json!({
        "MY_TEST_VARIABLE": "AAAAAAA"
    });

    let mut job = found_job
        .start_job(parameters)
        .await
        .expect("Job start failed");

        // Можно запустить пулинг для ожидания финальной ссылки, затем обновить сообщение
    // Ограничить продолжительность пулинга статуса 30 минутами
    info!("Job url pooling started for url: {}", job.get_info_api_url());
    
    use std::time::{
        Instant,
        Duration
    };
    let complete_time = Instant::now()
        .checked_add(Duration::from_secs(60 * 30))
        .expect("Complete time create failed");
    while complete_time > std::time::Instant::now() {

        actix_web::rt::time::delay_for(std::time::Duration::from_secs(10)).await;

        let result = job.try_to_get_real_job_url()
            .await
            .expect("Real job url request failed");

        match result {
            Some(real_url) => {
                info!("Real job url: {}", real_url);
                return;
            },
            None =>{
            }
        }
    }

    panic!("Real job url request failed")
}