#[macro_use] mod macros; // Специально самый первый
mod subtypes;
mod traits;
#[cfg(test)] mod tests;

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
