#![warn(clippy::all)]

use std::{
    process::{
        Command
    }
};

fn main() {   
    Command::new("clang")
                .spawn()
                .expect("spawn failed")
                .wait()
                .expect("wait failed");
}
