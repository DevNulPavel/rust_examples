#[macro_use] mod macros; // Специально самый первый
mod subtypes;
mod traits;
#[cfg(test)] mod tests;

use any_field_is_some::{
    AnyFieldIsSome
};
use self::{
    traits::{
        TryParseParams
    }
};
pub use self::{
    subtypes::{
        *
    }
};


#[derive(AnyFieldIsSome)]
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

    let is_some = params.any_field_is_some();
    if !is_some{
        panic!("Empty enviroment parameters");
    }

    params
}
