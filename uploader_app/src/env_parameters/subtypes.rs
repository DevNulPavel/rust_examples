use std::{
    env::{
        var
    }
};
use super::{
    traits::{
        EnvParams
    }
};
#[cfg(test)] 
use super::{
    traits::{
        EnvParamsTestable
    }
};
/*use any_derive::{
    AnyIsSome
};*/

/////////////////////////////////////////////////

env_params_type!(
    GitEnvironment{
        git_branch: "GIT_BRANCH", 
        git_commit: "GIT_COMMIT"
    }
);

/////////////////////////////////////////////////

env_params_type!(
    AmazonEnvironment{
        client_id: "AMAZON_CLIENT_ID", 
        client_secret: "AMAZON_CLIENT_SECRET",
        app_id: "AMAZON_APP_ID"
    }
);

/////////////////////////////////////////////////

env_params_type!(
    AppCenterEnvironment{
        token: "APP_CENTER_ACCESS_TOKEN",
        app: "APP_CENTER_APP_NAME",
        ownder: "APP_CENTER_APP_OWNER_NAME"
    }
);

/////////////////////////////////////////////////

env_params_type!(
    GooglePlayEnvironment{
        email: "GOOGLE_PLAY_SERVICE_EMAIL",
        key_id: "GOOGLE_PLAY_KEY_ID",
        key: "GOOGLE_PLAY_KEY"
    }
);

/////////////////////////////////////////////////

env_params_type!(
    GoogleDriveEnvironment{
        email: "GOOGLE_DRIVE_SERVICE_EMAIL",
        key_id: "GOOGLE_DRIVE_KEY_ID",
        key: "GOOGLE_DRIVE_KEY"
    }
);

/////////////////////////////////////////////////

env_params_type!(
    IOSEnvironment{
        user: "IOS_USER",
        pass: "IOS_PASS"
    }
);

/////////////////////////////////////////////////

pub struct SSHEnvironment{
    server: String,
    user: String,
    pass: Option<String>,
    key_file: Option<String>
}
impl EnvParams for SSHEnvironment {
    fn try_parse() -> Option<Self> {
        let server = var("SSH_SERVER").ok()?;
        let user = var("SSH_USER").ok()?;
        let pass = var("SSH_PASS").ok();
        let key_file = var("SSH_PRIVATE_KEY_PATH").ok();

        Some(SSHEnvironment{
            server,
            user,
            pass,
            key_file
        })
    }
    fn get_available_keys() -> &'static [&'static str] {
        let keys = &["SSH_SERVER",
                    "SSH_USER",
                    "SSH_PASS",
                    "SSH_PRIVATE_KEY_PATH"];
        keys
    }
}
#[cfg(test)]
impl EnvParamsTestable for SSHEnvironment {
    fn test(values: &std::collections::HashMap<String, String>){
        // TODO: Test
    }
}

/////////////////////////////////////////////////

env_params_type!(
    TargetSlackEnvironment{
        token: "TARGET_SLACK_API_TOKEN"
        // target: "TARGET_SLACK_CHANNEL_OR_USER_ID"
    }
);

/////////////////////////////////////////////////

pub struct ResultSlackEnvironment{
    token: String,
    text_prefix: Option<String>,
    channel: Option<String>,
    user_email: Option<String>,
    user_id: Option<String>
}
impl EnvParams for ResultSlackEnvironment {
    fn try_parse() -> Option<Self> {
        Some(ResultSlackEnvironment{
            token: var("RESULT_SLACK_API_TOKEN").ok()?,
            text_prefix: var("RESULT_SLACK_TEXT_PREFIX").ok(),
            channel: var("RESULT_SLACK_CHANNEL").ok(),
            user_id: var("RESULT_SLACK_USER").ok(),
            user_email: var("RESULT_SLACK_USER_EMAIL").ok()
        })
    }
    fn get_available_keys() -> &'static [&'static str] {
        let keys = &["RESULT_SLACK_API_TOKEN",
                     "RESULT_SLACK_TEXT_PREFIX",
                     "RESULT_SLACK_CHANNEL",
                     "RESULT_SLACK_USER",
                     "RESULT_SLACK_USER_EMAIL"];
        keys
    }
}
#[cfg(test)]
impl EnvParamsTestable for ResultSlackEnvironment {
    fn test(values: &std::collections::HashMap<String, String>){
        // TODO: Test
    }
}