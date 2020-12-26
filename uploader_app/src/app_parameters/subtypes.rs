
// commander.allowUnknownOption();


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

//////////////////////////////////////////////////////////////////////

params_data_type!(
    AmazonParams{
        Req{ 
            file_path : "amazon_input_file" : "Amazon uploading file"
        }
    }
);

//////////////////////////////////////////////////////////////////////

params_data_type!(
    AppCenterParams{
        Req{ 
            input_file : "app_center_input_file" : "App center input file"
        }
        Opt{ 
            symbols_file: "app_center_symbols_file" : "App center symbols file",
            build_description: "app_center_build_description": "App center build description"
        }
        MultOpt{
            distribution_groups: "app_center_distribution_groups": "App center distribution groups"
        }
    }
);