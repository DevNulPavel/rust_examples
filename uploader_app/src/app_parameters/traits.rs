
use clap::{
    App, 
    AppSettings, 
    Arg, 
    ArgMatches,
    ArgSettings
};

pub trait AppParams: Sized {
    fn get_args() -> Vec<Arg<'static, 'static>>;
    fn parse(values: &ArgMatches) -> Option<Self>;
}