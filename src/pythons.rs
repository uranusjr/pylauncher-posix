use std::cmp;
use std::fmt;
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
    let mut name = root.file_name()?.to_str()?;
    if name.starts_with("CPython-") {   // Pythonz prefixes CPython versions.
        name = &name[8..];
    }
    parse_version_from_name(name, 0)
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

#[derive(Eq)]
pub struct Python {
    location: String,
    version: (i16, i16, i16),
    order: usize,   // Tie breaker (e.g. order in PATH).
}

impl Python {
    pub fn from_managed(root: path::PathBuf, order: usize) -> Option<Python> {
        let p = root.join("bin/python");
        let version = parse_managed_root_path(&root)?;
        if p.is_file() {
            let location = String::from(p.to_str()?);
            Some(Self { location: location, version: version, order: order })
        } else {
            None
        }
    }

    pub fn from_in_path(p: path::PathBuf, order: usize) -> Option<Python> {
        let version = parse_executable_path(&p)?;
        let location = String::from(p.to_str()?);
        Some(Self { location: location, version: version, order: order })
    }

    pub fn matches(&self, spec: &specs::Spec) -> bool {
        match spec {
            specs::Spec::Major(x) => self.version.0 == (*x as i16),
            specs::Spec::Minor(x, y) => {
                self.version.0 == (*x as i16) && self.version.1 == (*y as i16)
            },
        }
    }

    pub fn location(&self) -> &str {
        self.location.as_str()
    }
}

impl PartialEq for Python {
    fn eq(&self, other: &Self) -> bool {
        self.version == other.version && self.order == other.order
    }
}

impl PartialOrd for Python {
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Python {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        match self.version.cmp(&other.version) {
            cmp::Ordering::Equal => {
                // Note that the order is reversed: The smaller the better.
                other.order.cmp(&self.order)
            },
            ordering => ordering,
        }
    }
}

impl fmt::Debug for Python {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Python {{ {:?}, ({}, {}, {}), {} }}",
            self.location,
            self.version.0, self.version.1, self.version.2,
            self.order)
    }
}
