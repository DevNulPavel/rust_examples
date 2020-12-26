
#[macro_use] mod macros;
mod traits;
mod subtypes;

use clap::{
    App, 
    AppSettings, 
    Arg, 
    ArgMatches,
    ArgSettings
};
use self::{
    traits::{
        AppParams
    }
};
pub use self::{
    subtypes::{
        *
    }
};

//////////////////////////////////////////////////////////////////////

pub struct AppParameters{
    pub amazon: Option<AmazonParams>,
    pub app_center: Option<AppCenterParams>
}

impl AppParameters{
    fn get_params_app<'a>(env_variables_help: Option<&'a str>) -> App<'a, 'a>{
        let app = App::new("Uploader application")
            .author("Pavel Ershov")
            .version("1.0.0")
            .setting(AppSettings::ColorAuto)
            .args(&AmazonParams::get_args())
            .args(&AppCenterParams::get_args());
    
        // Выводим кастомное описание окружения если надо
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
            amazon: AmazonParams::parse(&matches),
            app_center: AppCenterParams::parse(&matches)
        }
    }
    
    pub fn parse<T>(additional_help_provider: Option<T>) -> AppParameters
    where T: FnOnce()->String {
        let text = additional_help_provider
            .map(|func|{
                func()
            });

        let matches = AppParameters::get_params_app(text.as_deref())
            .get_matches();
    
        let parameters = AppParameters::matches_to_struct(matches);
        parameters
    }
}

#[cfg(test)]
mod tests{
    use super::*;

    #[test]
    fn test_app_parameters(){
        #![allow(non_upper_case_globals)]

        const app_center_file: &str = "app_center.apk";
        const app_center_symbols: &str = "app_center_symbols";
        const app_center_groups: &str = "group1,group2,group3";
        const app_center_description: &str = "TEST TEST test";

        let test_parameters = [
            "application",
            "--app_center_input_file", app_center_file,
            "--app_center_symbols_file", app_center_symbols,
            "--app_center_distribution_groups", app_center_groups,
            "--app_center_build_description", app_center_description
        ];

        let matches = AppParameters::get_params_app(None)
            .get_matches_from(&test_parameters);

        let result = AppParameters::matches_to_struct(matches);

        let ref app_center_params = result
            .app_center
            .expect("Appcenter values failed");

        assert_eq!(app_center_params.input_file.eq(app_center_file), true);
        assert_eq!(app_center_params.symbols_file, Some(app_center_symbols.to_owned()));
        assert_eq!(app_center_params.distribution_groups, Some(vec![
            "group1".to_owned(),
            "group2".to_owned(),
            "group3".to_owned(),
        ]));
        assert_eq!(app_center_params.build_description, Some(app_center_description.to_owned()));
    }
}