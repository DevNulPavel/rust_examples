use std::{
    env::{
        var
    }
};
use super::{
    traits::{
        TryParseParams
    }
};
#[cfg(test)] 
use super::{
    traits::{
        TestableParams
    }
};
/*use any_derive::{
    AnyIsSome
};*/

/////////////////////////////////////////////////

data_type!(GitParameters, 
    git_branch: "GIT_BRANCH", 
    git_commit: "GIT_COMMIT"
);

/////////////////////////////////////////////////

data_type!(AmazonParameters, 
    client_id: "AMAZON_CLIENT_ID", 
    client_secret: "AMAZON_CLIENT_SECRET",
    app_id: "AMAZON_APP_ID"
);

/////////////////////////////////////////////////

data_type!(AppCenterParameters,
    token: "APP_CENTER_ACCESS_TOKEN",
    app: "APP_CENTER_APP_NAME",
    ownder: "APP_CENTER_APP_OWNER_NAME"
);

/////////////////////////////////////////////////

data_type!(GooglePlayParameters,
    email: "GOOGLE_PLAY_SERVICE_EMAIL",
    key_id: "GOOGLE_PLAY_KEY_ID",
    key: "GOOGLE_PLAY_KEY"
);

/////////////////////////////////////////////////

data_type!(GoogleDriveParameters,
    email: "GOOGLE_DRIVE_SERVICE_EMAIL",
    key_id: "GOOGLE_DRIVE_KEY_ID",
    key: "GOOGLE_DRIVE_KEY"
);

/////////////////////////////////////////////////

data_type!(IOSParameters,
    user: "IOS_USER",
    pass: "IOS_PASS"
);

/////////////////////////////////////////////////

pub struct SSHParameters{
    server: String,
    user: String,
    pass: Option<String>,
    key_file: Option<String>
}
impl TryParseParams for SSHParameters {
    fn try_parse() -> Option<Self> {
        let server = var("SSH_SERVER").ok()?;
        let user = var("SSH_USER").ok()?;
        let pass = var("SSH_PASS").ok();
        let key_file = var("SSH_PRIVATE_KEY_PATH").ok();

        Some(SSHParameters{
            server,
            user,
            pass,
            key_file
        })
    }
}
#[cfg(test)]
impl TestableParams for SSHParameters {
    fn get_keys() -> &'static [&'static str] {
        let keys = &["SSH_SERVER",
                    "SSH_USER",
                    "SSH_PASS",
                    "SSH_PRIVATE_KEY_PATH"];
        keys
    }
    fn test(values: &std::collections::HashMap<String, String>){
        // TODO: Test
    }
}

/////////////////////////////////////////////////

data_type!(TargetSlackParameters, 
    token: "TARGET_SLACK_API_TOKEN",
    target: "TARGET_SLACK_CHANNEL_OR_USER_ID"
);

/////////////////////////////////////////////////

pub struct ResultSlackParameters{
    token: String,
    text_prefix: Option<String>,
    channel: Option<String>,
    user_email: Option<String>,
    user_id: Option<String>
}
impl TryParseParams for ResultSlackParameters {
    fn try_parse() -> Option<Self> {
        Some(ResultSlackParameters{
            token: var("RESULT_SLACK_API_TOKEN").ok()?,
            text_prefix: var("RESULT_SLACK_TEXT_PREFIX").ok(),
            channel: var("RESULT_SLACK_CHANNEL").ok(),
            user_id: var("RESULT_SLACK_USER").ok(),
            user_email: var("RESULT_SLACK_USER_EMAIL").ok()
        })
    }
}
#[cfg(test)]
impl TestableParams for ResultSlackParameters {
    fn get_keys() -> &'static [&'static str] {
        let keys = &["RESULT_SLACK_API_TOKEN",
                     "RESULT_SLACK_TEXT_PREFIX",
                     "RESULT_SLACK_CHANNEL",
                     "RESULT_SLACK_USER",
                     "RESULT_SLACK_USER_EMAIL"];
        keys
    }
    fn test(values: &std::collections::HashMap<String, String>){
        // TODO: Test
    }
}