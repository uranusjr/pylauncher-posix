#[macro_use] extern crate dbg;

mod confs;
mod finders;
mod procs;
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

fn resolve_spec(spec: Option<specs::Spec>) -> Option<specs::Spec> {
    let conf = confs::Conf::new();

    match spec {
        Some(specs::Spec::Minor(_, _)) => { return spec; },
        Some(specs::Spec::Major(x)) => match conf.default_python_for(x) {
            None => { return spec; },
            value => value,
        },
        None => conf.default_python(),
    }.and_then(|s| specs::parse_spec(format!("-{}", s).as_bytes()))
    // Big hack: The spec parser is used to parse options, so we add a dash
    // in front to fake it. LOL.
}

fn find_python(spec: Option<specs::Spec>) -> Option<String> {
    // PEP 486: If the invocation is without a spec, try to be aware of the
    // surrounding virtual environment.
    if spec.is_none() {
        match finders::get_virtual() {
            Some(python) => { return Some(python); }
            None => {},
        }
    }

    // Expand the spec based on configuration.
    let spec = resolve_spec(spec);

    // Find it!
    let found = match spec {
        Some(ref spec) => finders::find(spec),
        None => finders::find_best(),
    };
    if found.is_some() {
        return found;
    }

    // At this point the specified Python is not found. Report.
    match spec {
        None => { eprint!("Python"); },
        Some(spec) => {
            eprint!("Requested Python version (");
            match spec {
                specs::Spec::Major(x) => { eprint!("{}", x); },
                specs::Spec::Minor(x, y) => { eprint!("{}.{}", x, y) },
            }
            eprint!(")");
        }
    }
    eprint!(" is not installed\n");
    None
}

fn main() {
    let mut args = env::args();
    let prog = args.next().unwrap_or_default();

    let spec = match get_invocation() {
        Invocation::Help => { print_help!(prog); None },
        Invocation::Default => None,
        Invocation::Spec(spec) => { args.next(); Some(spec) },
    };

    let python = match find_python(spec) {
        Some(python) => python,
        None => { process::exit(-1); },
    };
    let args = args.collect();
    procs::run(&python, args);
}
