// use lalrpop::Configuration;
use lalrpop::process_root;

fn main() {
    // Либо просто можем руками вызывать:
    // - cargo install lalrpop
    // - lalrpop file.lalrpop

    process_root().unwrap();

    /*Configuration::new()
        .always_use_colors()
        .set_in_dir("src/lalrpop_source")
        .set_out_dir("src/lalrpop_result")
        .process()
        .unwrap();*/
}
