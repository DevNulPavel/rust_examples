use macroses::lazy_static;
use regex::Regex;

// lazy_static! {
//     static ref WARNING_TEST_NATIVE: Regex = {
//         println!("Compiling username regex...");
//         Regex::new("^[a-z0-9_-]{3,16}$").unwrap()
//     };
// }

// lazy_static! {
//     static ref ERROR_TEST_NATIVE: Regex = Regex::new("^[a-z0-9_-]{3,16}$").unwrap();
// }

// lazy_static! {
//     static ref WARNING_TEST_LIB: Regex = Regex::new("^[a-z0-9_-]{3,16}$").unwrap();
// }

// lazy_static! {
//     static ref ERROR_TEST_LIB: Regex = Regex::new("^[a-z0-9_-]{3,16}$").unwrap();
// }

lazy_static! {
    static ref REGEX_VAR: Regex = Regex::new("^[a-z0-9_-]{3,16}$").unwrap();
}

fn validate(name: &str) {
    // The USERNAME regex is compiled lazily the first time its value is accessed.
    println!(
        "is_match({:?}): {}",
        name,
        REGEX_VAR.is_match(name)
    );
}

fn main() {
    validate("fergie");
    validate("will.i.am");
}
