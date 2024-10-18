use crate::cli::common::{
    IggyCmdCommand, IggyCmdTest, IggyCmdTestCase, TestHelpCmd, TestStreamId, CLAP_INDENT,
    USAGE_PREFIX,
};
use assert_cmd::assert::Assert;
use async_trait::async_trait;
use iggy::client::Client;
use predicates::str::{contains, starts_with};
use serial_test::parallel;

struct TestStreamGetCmd {
    stream_id: u32,
    name: String,
    using_identifier: TestStreamId,
}

impl TestStreamGetCmd {
    fn new(stream_id: u32, name: String, using_identifier: TestStreamId) -> Self {
        Self {
            stream_id,
            name,
            using_identifier,
        }
    }

    fn to_arg(&self) -> String {
        match self.using_identifier {
            TestStreamId::Named => self.name.clone(),
            TestStreamId::Numeric => format!("{}", self.stream_id),
        }
    }
}

#[async_trait]
impl IggyCmdTestCase for TestStreamGetCmd {
    async fn prepare_server_state(&mut self, client: &dyn Client) {
        let stream = client.create_stream(&self.name, Some(self.stream_id)).await;
        assert!(stream.is_ok());
    }

    fn get_command(&self) -> IggyCmdCommand {
        IggyCmdCommand::new()
            .arg("stream")
            .arg("get")
            .arg(self.to_arg())
            .with_env_credentials()
    }

    fn verify_command(&self, command_state: Assert) {
        let start_message = match self.using_identifier {
            TestStreamId::Named => format!("Executing get stream with ID: {}\n", self.name.clone()),
            TestStreamId::Numeric => format!("Executing get stream with ID: {}\n", self.stream_id),
        };

        command_state
            .success()
            .stdout(starts_with(start_message))
            .stdout(contains(format!(
                "Stream ID            | {}",
                self.stream_id
            )))
            .stdout(contains(format!("Stream name          | {}", self.name)))
            .stdout(contains("Stream size          | 0"))
            .stdout(contains("Stream message count | 0"))
            .stdout(contains("Stream topics count  | 0"));
    }

    async fn verify_server_state(&self, _client: &dyn Client) {}
}

#[tokio::test]
#[parallel]
pub async fn should_be_successful() {
    let mut iggy_cmd_test = IggyCmdTest::default();

    iggy_cmd_test.setup().await;
    iggy_cmd_test
        .execute_test(TestStreamGetCmd::new(
            1,
            String::from("production"),
            TestStreamId::Named,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestStreamGetCmd::new(
            2,
            String::from("testing"),
            TestStreamId::Numeric,
        ))
        .await;
}

#[tokio::test]
#[parallel]
pub async fn should_help_match() {
    let mut iggy_cmd_test = IggyCmdTest::help_message();

    iggy_cmd_test
        .execute_test_for_help_command(TestHelpCmd::new(
            vec!["stream", "get", "--help"],
            format!(
                r#"Get details of a single stream with given ID

Stream ID can be specified as a stream name or ID

Examples:
 iggy stream get 1
 iggy stream get test

{USAGE_PREFIX} stream get <STREAM_ID>

Arguments:
  <STREAM_ID>
          Stream ID to get
{CLAP_INDENT}
          Stream ID can be specified as a stream name or ID

Options:
  -h, --help
          Print help (see a summary with '-h')
"#,
            ),
        ))
        .await;
}

#[tokio::test]
#[parallel]
pub async fn should_short_help_match() {
    let mut iggy_cmd_test = IggyCmdTest::default();

    iggy_cmd_test
        .execute_test_for_help_command(TestHelpCmd::new(
            vec!["stream", "get", "-h"],
            format!(
                r#"Get details of a single stream with given ID

{USAGE_PREFIX} stream get <STREAM_ID>

Arguments:
  <STREAM_ID>  Stream ID to get

Options:
  -h, --help  Print help (see more with '--help')
"#,
            ),
        ))
        .await;
}
