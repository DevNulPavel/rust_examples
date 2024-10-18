use crate::cli::common::{
    IggyCmdCommand, IggyCmdTest, IggyCmdTestCase, OutputFormat, TestHelpCmd, CLAP_INDENT,
    USAGE_PREFIX,
};
use assert_cmd::assert::Assert;
use async_trait::async_trait;
use iggy::client::Client;
use iggy::models::user_status::UserStatus;
use predicates::str::{contains, starts_with};
use serial_test::parallel;

struct TestUserListCmd {
    username: String,
    status: UserStatus,
    output: OutputFormat,
}

impl TestUserListCmd {
    fn new(username: String, status: UserStatus, output: OutputFormat) -> Self {
        Self {
            username,
            status,
            output,
        }
    }

    fn to_args(&self) -> Vec<&str> {
        self.output.to_args()
    }
}

#[async_trait]
impl IggyCmdTestCase for TestUserListCmd {
    async fn prepare_server_state(&mut self, client: &dyn Client) {
        let stream = client
            .create_user(&self.username, "secret", self.status, None)
            .await;
        assert!(stream.is_ok());
    }

    fn get_command(&self) -> IggyCmdCommand {
        IggyCmdCommand::new()
            .arg("user")
            .arg("list")
            .args(self.to_args())
            .with_env_credentials()
    }

    fn verify_command(&self, command_state: Assert) {
        command_state
            .success()
            .stdout(starts_with(format!(
                "Executing list users in {} mode",
                self.output
            )))
            .stdout(contains(self.username.clone()))
            .stdout(contains(self.status.to_string()));
    }

    async fn verify_server_state(&self, _client: &dyn Client) {}
}

#[tokio::test]
#[parallel]
pub async fn should_be_successful() {
    let mut iggy_cmd_test = IggyCmdTest::default();

    iggy_cmd_test.setup().await;
    iggy_cmd_test
        .execute_test(TestUserListCmd::new(
            String::from("producer"),
            UserStatus::Active,
            OutputFormat::Default,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestUserListCmd::new(
            String::from("testing"),
            UserStatus::Inactive,
            OutputFormat::List,
        ))
        .await;
    iggy_cmd_test
        .execute_test(TestUserListCmd::new(
            String::from("misc"),
            UserStatus::Active,
            OutputFormat::Table,
        ))
        .await;
}

#[tokio::test]
#[parallel]
pub async fn should_help_match() {
    let mut iggy_cmd_test = IggyCmdTest::help_message();

    iggy_cmd_test
        .execute_test_for_help_command(TestHelpCmd::new(
            vec!["user", "list", "--help"],
            format!(
                r#"List all users

Examples:
 iggy user list
 iggy user list --list-mode table
 iggy user list -l table

{USAGE_PREFIX} user list [OPTIONS]

Options:
  -l, --list-mode <LIST_MODE>
          List mode (table or list)
{CLAP_INDENT}
          [default: table]
          [possible values: table, list]

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
            vec!["user", "list", "-h"],
            format!(
                r#"List all users

{USAGE_PREFIX} user list [OPTIONS]

Options:
  -l, --list-mode <LIST_MODE>  List mode (table or list) [default: table] [possible values: table, list]
  -h, --help                   Print help (see more with '--help')
"#,
            ),
        ))
        .await;
}
