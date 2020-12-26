use uploader_app::{
    app_parameters::{
        AppParameters
    },
    env_parameters::{
        AppEnvValues
    }
};

fn main() {
    let possible_env_variables = AppEnvValues::get_possible_env_variables();
    let _ = AppParameters::parse(Some(possible_env_variables));
}
