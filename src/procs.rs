#[cfg(not(windows))] extern crate exec;

use std::process;

#[cfg(not(windows))]
pub fn run(command: &str, args: Vec<String>) {
    let err = exec::Command::new(command).args(&args).exec();
    eprintln!("Error: {}", err);
    process::exit(-1);
}

#[cfg(windows)]
pub fn run(command: &str, args: Vec<String>) {
    eprintln!("command = {:?}, arguments = {:?}", command, args);
    process::exit(-1);
}
