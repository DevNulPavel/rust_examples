#[macro_use] mod macros; // Специально самый первый
mod subtypes;
mod traits;
#[cfg(test)] mod tests;

use any_field_is_some::{
    AnyFieldIsSome
};
use self::{
    traits::{
        EnvParams
    }
};
pub use self::{
    subtypes::{
        *
    }
};

// TODO: Clap поддерживает и переменные окружения как оказалось

macro_rules! describe_env_values {
    ( $( $val: ident: $type_id:ident ),* ) => {
        #[derive(AnyFieldIsSome)]
        pub struct AppEnvValues{
            $( pub $val: Option<$type_id> ),*
        }
        impl AppEnvValues {
            pub fn parse() -> AppEnvValues {
                let params = AppEnvValues{
                    $( $val: $type_id::try_parse() ),*
                };
            
                let is_some = params.any_field_is_some();
                if !is_some{
                    panic!("Empty enviroment Environment");
                }
            
                params
            }
            pub fn get_possible_env_variables() -> Vec<&'static str>{
                // TODO: Убрать for, сделать на операторах
                let mut vec = Vec::new();
                $(
                    for key in $type_id::get_available_keys(){
                        vec.push(*key);
                    }
                )*
                vec
            }
        }
    };
}

describe_env_values!(
    git: GitEnvironment,
    amazon: AmazonEnvironment,
    app_center: AppCenterEnvironment,
    google_play: GooglePlayEnvironment,
    google_drive: GoogleDriveEnvironment,
    ios: IOSEnvironment,
    ssh: SSHEnvironment,
    target_slack: TargetSlackEnvironment,
    result_slack: ResultSlackEnvironment
);