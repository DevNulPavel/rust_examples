#![warn(clippy::all)]

use std::{
    process::{
        Command
    }
};

fn main() {
    let mut args_iter = std::env::args().skip(1);
    let jobs_count = args_iter
        .next()
        .expect("No jobs count")
        .parse::<usize>()
        .expect("Jobs count parse failed");
    let iter_count = args_iter
        .next()
        .expect("No iter count")
        .parse::<usize>()
        .expect("Iter count parse failed");
    let command = args_iter
        .next()
        .expect("No command");
    
    let repeat_per_thread = iter_count / jobs_count;
    let threads: Vec<std::thread::JoinHandle<_>> = std::iter::repeat(command.clone())
        .take(jobs_count)
        .map(|command|{
            let t = std::thread::spawn(move || {
                for _ in 0..repeat_per_thread {
                    let _ = match Command::new(&command).output(){
                        Ok(res) => res ,
                        Err(e) => {
                            panic!("Spawn command failed: {}", e)
                        }
                    };
                }
            });
            t
        })
        .collect();
    
    for t in threads{
        t.join().expect("Join failed");
    }
}
