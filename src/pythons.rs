use std::env;
use std::fs;
use std::path;

use specs;

// Major, minor, patch. Any component could be -1 if unknown.
// "patch" specifies the default patch number to use if the name is X.Y. This
// is needed because older Python versions use e.g. "3.2" to refer 3.2.0, but
// an executable "python3.2" is 3.2 with unknown patch name.
fn parse_version_from_name(name: &str, patch: i16) -> Option<(i16, i16, i16)> {
    let mut bytes = name.as_bytes().iter();

    let major = match specs::parse_spec_part(&mut bytes) {
        specs::SpecPart::Invalid => { return None; },
        specs::SpecPart::Number(n) => { return Some((n, -1, -1)) },
        specs::SpecPart::NumberDot(n) => n,
    };
    let minor = match specs::parse_spec_part(&mut bytes) {
        specs::SpecPart::Invalid => { return None; },
        specs::SpecPart::Number(n) => { return Some((major, n, patch)) },
        specs::SpecPart::NumberDot(n) => n,
    };
    match specs::parse_spec_part(&mut bytes) {
        specs::SpecPart::Invalid | specs::SpecPart::NumberDot(_) => None,
        specs::SpecPart::Number(n) => Some((major, minor, n)),
    }
}

fn parse_managed_root_path(root: &path::Path) -> Option<(i16, i16, i16)> {
    parse_version_from_name(root.file_name()?.to_str()?, 0)
}

fn parse_executable_path(location: &path::Path) -> Option<(i16, i16, i16)> {
    let name = location.file_name()?.to_str()?;
    if !name.starts_with("python") {
        return None;
    }
    // Special case: If the executable name is exactly "python", this is an
    // valid Python interpreter with an unspecified version.
    if name.len() == 6 {
        Some((-1, -1, -1))
    } else {
        parse_version_from_name(&name[6..], -1)
    }
}

#[derive(Debug)]
struct Python {
    location: String,
    version: (i16, i16, i16),
    order: usize,   // Tie breaker (e.g. order in PATH).
}

impl Python {
    fn from_managed(root: path::PathBuf) -> Option<Python> {
        let p = root.join("bin/python");
        let version = parse_managed_root_path(&root)?;
        if p.is_file() {
            let location = String::from(p.to_str()?);
            Some(Self { location: location, version: version, order: 0 })
        } else {
            None
        }
    }

    fn from_in_path(p: path::PathBuf, order: usize) -> Option<Python> {
        let version = parse_executable_path(&p)?;
        let location = String::from(p.to_str()?);
        Some(Self { location: location, version: version, order: order + 1 })
    }

    fn matches(&self, spec: &specs::Spec) -> bool {
        match spec {
            specs::Spec::Major(x) => self.version.0 == (*x as i16),
            specs::Spec::Minor(x, y) => {
                self.version.0 == (*x as i16) && self.version.1 == (*y as i16)
            },
        }
    }
}

struct ManagedFinder {
    dir: Option<fs::ReadDir>,
}

impl ManagedFinder {
    fn from(env_key: &str, subpath: &str) -> Self {
        match env::var(env_key) {
            Ok(v) => {
                let mut dir = path::PathBuf::from(v);
                dir.push(subpath);
                Self { dir: fs::read_dir(dir).ok() }
            },
            Err(_) => Self { dir: None },
        }
    }
}

impl Iterator for ManagedFinder {
    type Item = Python;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let prefix = match self.dir {
                None => { return None; },
                Some(ref mut d) => d.next()?.ok()?.path(),
            };
            match Python::from_managed(prefix) {
                None => {},
                Some(python) => { return Some(python); },
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
            let prefix = match self.dir {
                None => { return None; },
                Some(ref mut d) => d.next()?.ok()?.path(),
            };
            match Python::from_in_path(prefix, self.order) {
                None => {},
                Some(python) => { return Some(python); },
            }
        }
    }
}

fn collect_all() -> Vec<Python> {
    let mut pythons = vec![];

    // TODO: Maybe we want to switch to explicit setup, i.e. expose an
    // environment variable. An example set up would be:
    // PY_PYTHON_MANAGED=$PYENV/versions:$ASDF_DATA_DIR/installs/python
    pythons.extend(ManagedFinder::from("ASDF_DATA_DIR", "installs/python"));
    pythons.extend(ManagedFinder::from("PYENV_ROOT", "versions"));

    let path = env::var("PATH").unwrap_or_default();
    for (i, dir) in env::split_paths(&path).enumerate() {
        pythons.extend(ExecutableFinder::from(dir, i));
    }

    pythons
}

fn select_best(best: Option<Python>, next: Python) -> Option<Python> {
    // Current ordering method:
    // 1. Highest specified version possible.
    // 2. Managed over PATH-exposed.
    // 3. Order by PATH order.
    // This logic has some unique quirks, e.g. if PATH[0] has python3, but
    // PATH[1] has python3.7, py -3 would select python3.7 (Rule 1, 3.7 is
    // more specified than 3). This could potentially change in the future.
    match best {
        None => Some(next),
        Some(p) => if next.version > p.version || next.order < p.order {
            Some(next)
        } else {
            Some(p)
        },
    }
}

pub fn find(spec: &specs::Spec) -> Option<String> {
    collect_all().into_iter()
        .filter(|python| python.matches(spec))
        .fold(None, select_best)
        .map(|p| p.location)
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
    collect_all().into_iter().fold(None, select_best).map(|p| p.location)
}
