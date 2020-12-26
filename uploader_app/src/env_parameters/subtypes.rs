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
        Req{
            git_branch: "GIT_BRANCH", 
            git_commit: "GIT_COMMIT"
        }
    }
);

/////////////////////////////////////////////////

env_params_type!(
    AmazonEnvironment{
        Req{
            client_id: "AMAZON_CLIENT_ID", 
            client_secret: "AMAZON_CLIENT_SECRET",
            app_id: "AMAZON_APP_ID"
        }
    }
);

/////////////////////////////////////////////////

env_params_type!(
    AppCenterEnvironment{
        Req{
            token: "APP_CENTER_ACCESS_TOKEN",
            app: "APP_CENTER_APP_NAME",
            ownder: "APP_CENTER_APP_OWNER_NAME"
        }
    }
);

/////////////////////////////////////////////////

env_params_type!(
    GooglePlayEnvironment{
        Req{
            email: "GOOGLE_PLAY_SERVICE_EMAIL",
            key_id: "GOOGLE_PLAY_KEY_ID",
            key: "GOOGLE_PLAY_KEY"
        }
    }
);

/////////////////////////////////////////////////

env_params_type!(
    GoogleDriveEnvironment{
        Req{
            email: "GOOGLE_DRIVE_SERVICE_EMAIL",
            key_id: "GOOGLE_DRIVE_KEY_ID",
            key: "GOOGLE_DRIVE_KEY"
        }
    }
);

/////////////////////////////////////////////////

env_params_type!(
    IOSEnvironment{
        Req{
            user: "IOS_USER",
            pass: "IOS_PASS"
        }
    }
);

/////////////////////////////////////////////////

env_params_type!(
    SSHEnvironment{
        Req{
            server: "SSH_SERVER",
            user: "SSH_USER",
            pass: "SSH_PASS"
        }
        Opt{ 
            key_file: "SSH_PRIVATE_KEY_PATH" 
        }
    }
);

/////////////////////////////////////////////////

env_params_type!(
    TargetSlackEnvironment{
        Req{
            token: "TARGET_SLACK_API_TOKEN"
        }
    }
);

/////////////////////////////////////////////////

env_params_type!(
    ResultSlackEnvironment{
        Req{
            token: "RESULT_SLACK_API_TOKEN"
        }
        Opt{
            text_prefix: "RESULT_SLACK_TEXT_PREFIX",
            channel: "RESULT_SLACK_CHANNEL",
            user_id: "RESULT_SLACK_USER",
            user_email: "RESULT_SLACK_USER_EMAIL"
        }
    }
);