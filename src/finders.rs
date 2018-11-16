use std::env;
use std::fs;
use std::path;

use pythons::Python;
use specs;

struct ManagedFinder {
    dir: Option<fs::ReadDir>,
    order: usize,
}

impl ManagedFinder {
    fn from(dir: path::PathBuf, order: usize) -> Self {
        Self { dir: fs::read_dir(dir).ok(), order: order }
    }
}

impl Iterator for ManagedFinder {
    type Item = Python;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let prefix = self.dir.as_mut()?.next()?.ok()?.path();
            match Python::from_managed(prefix, self.order) {
                None => {},
                Some(python) => {
                    dbg!("Managed" => &python);
                    return Some(python);
                },
            }
        }
    }
}

struct ExecutableFinder {
    dir: Option<fs::ReadDir>,
    order: usize,
}

impl ExecutableFinder {
    fn from(dir: path::PathBuf, order: usize) -> Self {
        Self { dir: fs::read_dir(dir).ok(), order: order }
    }
}

impl Iterator for ExecutableFinder {
    type Item = Python;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let prefix = self.dir.as_mut()?.next()?.ok()?.path();
            match Python::from_in_path(prefix, self.order) {
                None => {},
                Some(python) => {
                    dbg!("Executable" => &python);
                    return Some(python);
                },
            }
        }
    }
}

fn collect_all() -> Vec<Python> {
    let mut pythons = vec![];
    let mut order = 0;

    match env::var("PY_MANAGED_DIR") {
        Err(_) => {},
        Ok(value) => {
            for dir in env::split_paths(&value) {
                pythons.extend(ManagedFinder::from(dir, order));
                order += 1;
            }
        },
    }

    match env::var("PATH") {
        Err(_) => {},
        Ok(value) => {
            for dir in env::split_paths(&value) {
                pythons.extend(ExecutableFinder::from(dir, order));
                order += 1;
            }
        },
    }

    pythons
}

fn select_best(best: Option<Python>, next: Python) -> Option<Python> {
    Some(match best {
        None => next,
        Some(p) => if next > p { next } else { p },
    })
}

pub fn find(spec: &specs::Spec) -> Option<String> {
    collect_all().into_iter()
        .filter(|python| python.matches(spec))
        .fold(None, select_best)
        .map(|p| p.location().to_string())
}


fn get_virtual() -> Option<String> {
    // The Windows launcher seems to swallow all errors, so we're not worse.
    let root = match env::var("VIRTUAL_ENV") {
        Ok(v) => v,
        Err(_) => { return None; },
    };

    let location = path::Path::new(&root).join("bin/python");
    if location.is_file() {
        location.to_str().map(String::from)
    } else {
        None
    }
}

pub fn find_default() -> Option<String> {
    match get_virtual() {
        Some(v) => { return Some(v); },
        None => {},
    }
    collect_all().into_iter()
        .fold(None, select_best)
        .map(|p| p.location().to_string())
}
