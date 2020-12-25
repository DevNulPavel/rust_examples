use std::{
    env::{
        var
    },
    path::{
        PathBuf
    }
};
/*use any_derive::{
    AnyIsSome
};*/

// https://doc.rust-lang.org/reference/macros-by-example.html
// https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros
// https://doc.rust-lang.org/book/ch19-06-macros.html
// https://doc.rust-lang.org/edition-guide/rust-2018/macros/custom-derive.html
// https://doc.rust-lang.org/book/ch19-06-macros.html#how-to-write-a-custom-derive-macro
macro_rules! data_type {
    ($type: ident, $($val: ident: $key: literal),*) => {
        pub struct $type {
            $($val: String,)*
        }
        impl TryParseParams for $type {
            fn try_parse() -> Option<Self> {
                Some(Self{
                    $($val: var($key).ok()?),*
                })
            }
        }
        #[cfg(test)]
        impl TestableParams for $type {
            fn test(values: &std::collections::HashMap<String, String>){
                let val = Self::try_parse()
                    .expect(&format!("Failed to parse: {}", stringify!($type)));
                $( assert_eq!(val.$val.eq(&values[$key]), true); )*
            }
            fn get_keys() -> &'static [&'static str] {
                let keys = &[
                    $($key,)*
                ];
                keys
            }
        }
    };
}

/////////////////////////////////////////////////

trait TryParseParams: Sized {
    fn try_parse() -> Option<Self>;
}

#[cfg(test)]
trait TestableParams: Sized {
    fn get_keys() -> &'static [&'static str];
    fn test(values: &std::collections::HashMap<String, String>);
}


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


/////////////////////////////////////////////////

// #[derive(AnyIsSome)]
pub struct EnvParameters{
    git: Option<GitParameters>,
    amazon: Option<AmazonParameters>,
    app_center: Option<AppCenterParameters>,
    google_play: Option<GooglePlayParameters>,
    google_drive: Option<GoogleDriveParameters>,
    ios: Option<IOSParameters>,
    ssh: Option<SSHParameters>,
    target_slack: Option<TargetSlackParameters>,
    result_slack: Option<ResultSlackParameters>,
}

pub fn parse() -> EnvParameters {
    let params = EnvParameters{
        git: GitParameters::try_parse(),
        amazon: AmazonParameters::try_parse(),
        app_center: AppCenterParameters::try_parse(),
        google_play: GooglePlayParameters::try_parse(),
        google_drive: GoogleDriveParameters::try_parse(),
        ios: IOSParameters::try_parse(),
        ssh: SSHParameters::try_parse(),
        target_slack: TargetSlackParameters::try_parse(),
        result_slack: ResultSlackParameters::try_parse()
    };

    // TODO: Проверить валидность

    params
}


#[cfg(test)]
mod tests{
    use super::*;
    use std::{
        env::{
            set_var
        },
        collections::{
            HashMap
        }
    };
    use rand::{
        distributions::{
            Alphanumeric
        },
        thread_rng, 
        Rng
    };

    macro_rules! test_type_before {
        ($map: ident, $type: ident) => {
            $map.extend(get_random_key_values::<$type>().into_iter());
        };
    }
    
    macro_rules! test_type_after {
        ($map: ident, $type: ident) => {
            $type::test(&$map);
        };
    }

    macro_rules! test_types {
        ($($type: ident),*) => {
            let mut test_values: HashMap<String, String> = HashMap::new();

            $( test_type_before!(test_values, $type);)*

            test_values
                .iter()
                .for_each(|(k, v)|{
                    set_var(k, v);
                });

            $( test_type_after!(test_values, $type); )*
        };
    }

    fn rand_string() -> String{
        let rand_string: Vec<u8> = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(10)
            .collect();
        
        std::str::from_utf8(&rand_string).unwrap().to_owned()
    }

    fn get_random_key_values<T: TestableParams>()-> HashMap<String, String>{
        let keys = T::get_keys();
        let res = keys
            .iter()
            .fold(HashMap::new(), |mut prev, key|{
                let key = key.to_string();
                prev.insert(key, rand_string());
                prev
            });
        res
    }

    #[test]
    fn test_env_parameters(){
        test_types! (
            GitParameters,
            AmazonParameters,
            AppCenterParameters,
            GooglePlayParameters,
            GoogleDriveParameters,
            IOSParameters,
            SSHParameters,
            TargetSlackParameters,
            ResultSlackParameters
        );
    }
}