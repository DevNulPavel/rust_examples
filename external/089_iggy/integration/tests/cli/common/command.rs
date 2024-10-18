use iggy::users::defaults::{DEFAULT_ROOT_PASSWORD, DEFAULT_ROOT_USERNAME};
use std::collections::HashMap;

pub(crate) struct IggyCmdCommand {
    opts: Vec<String>,
    args: Vec<String>,
    env: HashMap<String, String>,
}

impl IggyCmdCommand {
    pub(crate) fn new() -> Self {
        Self {
            opts: vec![],
            args: vec![],
            env: HashMap::new(),
        }
    }

    pub(crate) fn opt(mut self, arg: impl Into<String>) -> Self {
        self.opts.push(arg.into());
        self
    }

    pub(crate) fn opts(mut self, opts: Vec<impl Into<String>>) -> Self {
        self.opts.append(
            opts.into_iter()
                .map(|a| a.into())
                .collect::<Vec<_>>()
                .as_mut(),
        );
        self
    }

    pub(crate) fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub(crate) fn args(mut self, args: Vec<impl Into<String>>) -> Self {
        self.args.append(
            args.into_iter()
                .map(|a| a.into())
                .collect::<Vec<_>>()
                .as_mut(),
        );
        self
    }

    pub(crate) fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    pub(crate) fn with_env_credentials(self) -> Self {
        self.env("IGGY_USERNAME", DEFAULT_ROOT_USERNAME)
            .env("IGGY_PASSWORD", DEFAULT_ROOT_PASSWORD)
    }

    pub(crate) fn with_cli_credentials(mut self) -> Self {
        self.opts.push(String::from("--username"));
        self.opts.push(String::from(DEFAULT_ROOT_USERNAME));
        self.opts.push(String::from("--password"));
        self.opts.push(String::from(DEFAULT_ROOT_PASSWORD));

        self
    }

    pub(crate) fn get_opts_and_args(&self) -> Vec<String> {
        let mut cmd = vec![];
        cmd.append(&mut self.opts.clone());
        cmd.append(&mut self.args.clone());
        cmd
    }

    pub(crate) fn get_env(&self) -> HashMap<String, String> {
        self.env.clone()
    }

    pub(crate) fn disable_backtrace(self) -> Self {
        self.env("RUST_BACKTRACE", "0")
    }
}
