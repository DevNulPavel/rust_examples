// const commaSeparatedList = (value) => {
//     return value.split(",").filter((val)=>{
//         return val && (val.length > 0);
//     });
// };
// commander.allowUnknownOption();
// commander.option("--amazon_input_file <input apk>", "Input file for amazon uploading");
// commander.option("--app_center_input_file <input .apk or .ipa>", "Input file for app center uploading");
// commander.option("--app_center_symbols_file <input .dSYM.zip>", "Input symbols archive for app center uploading");
// commander.option("--app_center_distribution_groups <comma_separeted_groups>", "App center distribution groups: 'group1','group2'", commaSeparatedList);
// commander.option("--app_center_build_description <text>", "App center build description");
// commander.option("--google_drive_files <comma_separeted_file_paths>", "Input files for uploading: -gdrivefiles 'file1','file2'", commaSeparatedList);
// commander.option("--google_drive_target_folder_id <folder_id>", "Target Google drive folder ID");
// commander.option("--google_drive_target_subfolder_name <folder_name>", "Target Google drive subfolder name");
// commander.option("--google_drive_target_owner_email <email>", "Target Google drive folder owner email");
// commander.option("--google_drive_target_domain <domain>", "Target Google drive shared domain");
// commander.option("--google_play_upload_file <file_path>", "File path for google play uploading");
// commander.option("--google_play_target_track <target_track>", "Target track for google play build");
// commander.option("--google_play_package_name <package>", "Package name for google play uploading: com.gameinsight.gplay.island2");
// commander.option("--ipa_to_ios_app_store <ipa build path>", "Ipa file for iOS App store uploading");
// commander.option("--ssh_upload_files <comma_separeted_file_paths>", "Input files for uploading: -sshfiles='file1','file2'", commaSeparatedList);
// commander.option("--ssh_target_server_dir <dir>", "Target server directory for files");
// commander.option("--slack_upload_files <comma_separeted_file_paths>", "Input files for uploading: -slackfiles='file1','file2'", commaSeparatedList);
// commander.option("--slack_upload_channel <channel>", "Slack upload files channel");
// commander.option("--slack_user <user>", "Slack user name for direct messages");
// commander.option("--slack_user_email <user_email>", "Slack user email for direct messages");
// commander.option("--slack_user_text <text>", "Slack direct message text");
// commander.option("--slack_user_qr_commentary <text>", "Slack direct QR code commentary");
// commander.option("--slack_user_qr_text <text>", "Slack direct QR code content");
// commander.parse(process.argv);

use clap::{
    App, 
    AppSettings, 
    Arg, 
    ArgMatches
};

pub struct AmazonParams{
    path: String
}

pub struct AppParameters{
    //pub amazon: Option<AmazonParams>
}

fn get_params_app<'a>(env_variables_help: Option<&'a str>) -> App<'a, 'a> {
    let app = App::new("Uploader application")
        .author("Pavel Ershov")
        .version("1.0.0")
        .setting(AppSettings::ColorAuto)
        .arg(Arg::with_name("in_file")
                .long("amazon_input_file")
                .value_delimiter(",")
                .takes_value(true));

    // Выводим кастомное окружение если надо
    let app = match env_variables_help {
        Some(env_variables_text) => {
            app.after_help(env_variables_text)
        },
        None => {
            app
        }
    };
    
    app
}

fn matches_to_struct(matches: ArgMatches) -> AppParameters {
    AppParameters {
    }
}

pub fn parse(env_variables_help: Option<Vec<&str>>) -> AppParameters {
    let text = env_variables_help
        .map(|env_variables|{
            env_variables    
                .into_iter()
                .fold(String::from("ENVIRONMENT VARIABLES:\n"), |mut prev, var|{
                    prev.push_str("    - ");
                    prev.push_str(var);
                    prev.push_str("\n");
                    prev
                })
        });

    let matches = get_params_app(text.as_deref())
        .get_matches();

    let parameters = matches_to_struct(matches);
    parameters
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_app_parameters(){
        let test_parameters = [
            "asd",
            "asdsa"
        ];

        let matches = get_params_app(None)
            .get_matches_from(&test_parameters);

        let result = matches_to_struct(matches);
    }
}