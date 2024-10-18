use std::collections::HashMap;

use assert_cmd::assert::Assert;
use async_trait::async_trait;
use iggy::client::Client;
use iggy::{args::ArgsOptional, cli::context::common::ContextConfig};
use integration::test_server::TestServer;
use predicates::str::{contains, starts_with};
use serial_test::parallel;

use crate::cli::common::{IggyCmdCommand, IggyCmdTest, IggyCmdTestCase};

use super::common::TestIggyContext;

struct TestContextApplied {
    set_transport_context: Option<String>,
    set_transport_arg: Option<String>,
    test_iggy_context: TestIggyContext,
}

impl TestContextApplied {
    fn new(set_transport_context: Option<String>, set_transport_arg: Option<String>) -> Self {
        let test_iggy_context = TestIggyContext::new(
            Some(HashMap::from([
                (
                    "default".to_string(),
                    ContextConfig {
                        ..Default::default()
                    },
                ),
                (
                    "second".to_string(),
                    ContextConfig {
                        iggy: ArgsOptional {
                            transport: set_transport_context.clone(),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                ),
            ])),
            Some("second".to_string()),
        );

        Self {
            set_transport_context,
            set_transport_arg,
            test_iggy_context,
        }
    }
}

#[async_trait]
impl IggyCmdTestCase for TestContextApplied {
    async fn prepare_server_state(&mut self, _client: &dyn Client) {
        self.test_iggy_context.prepare().await;
    }

    fn protocol(&self, server: &TestServer) -> Vec<String> {
        let transport = self
            .set_transport_arg
            .as_ref()
            .or(self.set_transport_context.as_ref());

        match transport {
            Some(protocol) => match protocol.as_str() {
                "quic" => vec![
                    "--quic-server-address".into(),
                    server.get_quic_udp_addr().unwrap(),
                ],
                _ => vec![
                    "--tcp-server-address".into(),
                    server.get_raw_tcp_addr().unwrap(),
                ],
            },
            None => panic!("either set_transport_arg or set_transport_context must be set"),
        }
    }

    fn get_command(&self) -> IggyCmdCommand {
        let cmd = IggyCmdCommand::new()
            .env(
                "IGGY_HOME",
                self.test_iggy_context.get_iggy_home().to_str().unwrap(),
            )
            .with_env_credentials();

        let cmd = match &self.set_transport_arg {
            Some(protocol) => cmd.opts(vec!["--transport", protocol.as_str()]),
            None => cmd,
        };

        cmd.arg("me")
    }

    fn verify_command(&self, command_state: Assert) {
        let command_state = command_state
            .success()
            .stdout(starts_with("Executing me command\n"));

        match (&self.set_transport_arg, &self.set_transport_context) {
            // When both are set, the arg should override the context
            (Some(transport_arg), Some(_transport_context)) => {
                command_state.stdout(contains(format!(
                    "Transport | {}",
                    transport_arg.to_uppercase()
                )));
            }
            (None, Some(transport_context)) => {
                command_state.stdout(contains(format!(
                    "Transport | {}",
                    transport_context.to_uppercase()
                )));
            }
            _ => {}
        }
    }

    async fn verify_server_state(&self, _client: &dyn Client) {}
}

#[tokio::test]
#[parallel]
pub async fn should_apply_context() {
    let mut iggy_cmd_test = IggyCmdTest::new(true);
    iggy_cmd_test.setup().await;

    iggy_cmd_test
        .execute_test(TestContextApplied::new(Some("quic".to_string()), None))
        .await;
}

#[tokio::test]
#[parallel]
pub async fn should_allow_args_to_override_context() {
    let mut iggy_cmd_test = IggyCmdTest::new(true);
    iggy_cmd_test.setup().await;

    iggy_cmd_test
        .execute_test(TestContextApplied::new(
            Some("quic".to_string()),
            Some("tcp".to_string()),
        ))
        .await;
}
