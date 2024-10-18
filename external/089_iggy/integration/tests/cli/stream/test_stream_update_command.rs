use crate::cli::common::{
    IggyCmdCommand, IggyCmdTest, IggyCmdTestCase, TestHelpCmd, TestStreamId, CLAP_INDENT,
    USAGE_PREFIX,
};
use assert_cmd::assert::Assert;
use async_trait::async_trait;
use iggy::client::Client;
use predicates::str::diff;
use serial_test::parallel;

struct TestStreamUpdateCmd {
    stream_id: u32,
    name: String,
    new_name: String,
    using_identifier: TestStreamId,
}

impl TestStreamUpdateCmd {
    fn new(stream_id: u32, name: String, new_name: String, using_identifier: TestStreamId) -> Self {
        Self {
            stream_id,
            name,
            new_name,
            using_identifier,
        }
    }

    fn to_args(&self) -> Vec<String> {
        match self.using_identifier {
            TestStreamId::Named => vec![self.name.clone(), self.new_name.clone()],
            TestStreamId::Numeric => {
                vec![format!("{}", self.stream_id), self.new_name.clone()]
            }
        }
    }
}

#[async_trait]
impl IggyCmdTestCase for TestStreamUpdateCmd {
    async fn prepare_server_state(&mut self, client: &dyn Client) {
        let stream = client.create_stream(&self.name, Some(self.stream_id)).await;
        assert!(stream.is_ok());
    }

    fn get_command(&self) -> IggyCmdCommand {
        IggyCmdCommand::new()
            .arg("stream")
            .arg("update")
            .args(self.to_args())
            .with_env_credentials()
    }

    fn verify_command(&self, command_state: Assert) {
        let message = match self.using_identifier {
            TestStreamId::Named => format!("Executing update stream with ID: {} and name: {}\nStream with ID: {} updated name: {}\n", self.name, self.new_name, self.name, self.new_name),
            TestStreamId::Numeric => format!("Executing update stream with ID: {} and name: {}\nStream with ID: {} updated name: {}\n", self.stream_id, self.new_name, self.stream_id, self.new_name),
        };

        command_state.success().stdout(diff(message));
    }

    async fn verify_server_state(&self, client: &dyn Client) {
        let stream = client.get_stream(&self.stream_id.try_into().unwrap()).await;
        assert!(stream.is_ok());
        let stream = stream.unwrap().expect("Stream not found");
        assert_eq!(stream.name, self.new_name);
    }
}

#[tokio::test]
#[parallel]
pub async fn should_be_successful() {
    let mut iggy_cmd_test = IggyCmdTest::default();

    iggy_cmd_test.setup().await;
    iggy_cmd_test
        .execute_test(TestStreamUpdateCmd::new(
            1,
            String::from("testing"),
            String::from("development"),
            TestStreamId::Numeric,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestStreamUpdateCmd::new(
            2,
            String::from("production"),
            String::from("prototype"),
            TestStreamId::Named,
        ))
        .await;
}

#[tokio::test]
#[parallel]
pub async fn should_help_match() {
    let mut iggy_cmd_test = IggyCmdTest::help_message();

    iggy_cmd_test
        .execute_test_for_help_command(TestHelpCmd::new(
            vec!["stream", "update", "--help"],
            format!(
                r#"Update stream name for given stream ID

Stream ID can be specified as a stream name or ID

Examples:
 iggy stream update 1 production
 iggy stream update test development

{USAGE_PREFIX} stream update <STREAM_ID> <NAME>

Arguments:
  <STREAM_ID>
          Stream ID to update
{CLAP_INDENT}
          Stream ID can be specified as a stream name or ID

  <NAME>
          New name for the stream

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
            vec!["stream", "update", "-h"],
            format!(
                r#"Update stream name for given stream ID

{USAGE_PREFIX} stream update <STREAM_ID> <NAME>

Arguments:
  <STREAM_ID>  Stream ID to update
  <NAME>       New name for the stream

Options:
  -h, --help  Print help (see more with '--help')
"#,
            ),
        ))
        .await;
}
