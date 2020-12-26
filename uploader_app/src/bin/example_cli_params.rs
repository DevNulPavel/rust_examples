use uploader_app::{
    app_parameters::{
        AppParameters
    },
    env_parameters::{
        AppEnvValues
    }
};

fn main() {
    let _app_parameters = AppParameters::parse(Some(||{
        AppEnvValues::get_possible_env_variables()    
            .into_iter()
            .fold(String::from("ENVIRONMENT VARIABLES:\n"), |mut prev, var|{
                prev.push_str("    - ");
                prev.push_str(var);
                prev.push_str("\n");
                prev
            })
    }));
}
