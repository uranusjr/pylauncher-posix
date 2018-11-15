#[macro_use] extern crate dbg;
extern crate exec;

mod pythons;
mod specs;

use std::env;
use std::process;


#[derive(Debug)]
enum Invocation {
    Help,
    Default,
    Spec(specs::Spec),
}

fn get_invocation() -> Invocation {
    let mut args = env::args();
    args.next();   // Program name, don't care.

    let arg = match args.next() {
        None => {
            // No arguments -- Launch the default Python (without arguments).
            return Invocation::Default;
        },
        Some(arg) => arg,
    };

    // Only argument is "-h" or "--help" -- Show launcher and default help.
    if args.next().is_none() && (arg == "-h" || arg == "--help") {
        return Invocation::Help;
    }

    // Parse the first argument to determine whether what Python to launch.
    match specs::parse_spec(&mut arg.as_bytes()) {
        None => Invocation::Default,
        Some(spec) => Invocation::Spec(spec),
    }
}

// Help message format copied from Python Launcher for Windows.
macro_rules! print_help { ($prog: expr) => { println!("\
Python Launcher for POSIX Version {}

usage: {} \
[ launcher-arguments ] [ python-arguments ] script [ script-arguments ]

Launcher arguments:

-2     : Launch the latest Python 2.x version
-3     : Launch the latest Python 3.x version
-X.Y   : Launch the specified Python version

The following help text is from Python:
", env!("CARGO_PKG_VERSION"), $prog) } }

macro_rules! find_python {
    ( $spec: expr ) => {
        match pythons::find(&$spec) {
            Some(python) => python,
            None => {
                eprint!("Requested Python version (");
                match $spec {
                    specs::Spec::Major(x) => eprint!("{}", x),
                    specs::Spec::Minor(x, y) => eprint!("{}.{}", x, y),
                };
                eprint!(") is not installed\n");
                process::exit(-1);
            },
        }
    };

    () => {
        match pythons::find_default() {
            Some(python) => python,
            None => {
                eprintln!("Python is not installed");
                process::exit(-1);
            }
        }
    };
}

fn run_child(python: &str, args: Vec<String>) {
    let err = exec::Command::new(python).args(&args).exec();
    eprintln!("Error: {}", err);
    process::exit(-1);
}

fn main() {
    let mut args = env::args();
    let prog = args.next().unwrap_or_default();

    let inv = get_invocation();

    let python = match inv {
        Invocation::Help => { print_help!(prog); find_python!() },
        Invocation::Default => find_python!(),
        Invocation::Spec(spec) => { args.next(); find_python!(spec) },
    };

    let args = args.collect();
    run_child(&python, args);
}
