
use reqwest::{
    Client
};
use super::{
    JenkinsClient
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

    let client = JenkinsClient::new(request_client, &jenkins_user, &jenkins_api_token);

    let jobs = client.request_jenkins_jobs_list()
        .await
        .expect("Jobs list failed");

    let found_job = jobs
        .iter()
        .find(|job|{
            job.get_info().name == "utils-check-free-space"
        })
        .expect("Required job is not found");

    // TODO: ???
    let _found_parameters = found_job
        .request_jenkins_job_info()
        .await
        .expect("Job parameter request error");

    let parameters = serde_json::json!({
        "MY_TEST_VARIABLE": "AAAAAAA"
    });

    let _job_start_result = found_job
        .start_job(parameters)
        .await
        .expect("Job start failed");

    //assert_eq!(found_job.is_some(), true);
}