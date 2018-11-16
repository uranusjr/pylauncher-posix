use std::cmp;
use std::fmt;
use std::path;

use specs;

type Version = (i16, i16, i16);

enum VersionPad {
    Zero = 0,
    Unknown = -1,
}

// Major, minor, patch. Any component could be -1 if unknown.
// "pad" specifies the default number to use if a part is missing. This
// is needed because older Python versions use e.g. "3.2" to refer 3.2.0, but
// an executable "python3.2" is 3.2 with unknown patch name.
fn parse_version_from_name(name: &str, pad: VersionPad) -> Option<Version> {
    let pad = pad as i16;
    let mut bytes = name.as_bytes().iter();

    let major = match specs::parse_spec_part(&mut bytes) {
        specs::SpecPart::Invalid => { return None; },
        specs::SpecPart::Number(n) => { return Some((n, pad, pad)) },
        specs::SpecPart::NumberDot(n) => n,
    };
    let minor = match specs::parse_spec_part(&mut bytes) {
        specs::SpecPart::Invalid => { return None; },
        specs::SpecPart::Number(n) => { return Some((major, n, pad)) },
        specs::SpecPart::NumberDot(n) => n,
    };
    match specs::parse_spec_part(&mut bytes) {
        specs::SpecPart::Invalid | specs::SpecPart::NumberDot(_) => None,
        specs::SpecPart::Number(n) => Some((major, minor, n)),
    }
}

fn parse_managed_root_path(root: &path::Path) -> Option<Version> {
    let mut name = root.file_name()?.to_str()?;
    if name.starts_with("CPython-") {   // Pythonz prefixes CPython versions.
        name = &name[8..];
    }
    parse_version_from_name(name, VersionPad::Zero)
}

fn parse_executable_path(location: &path::Path) -> Option<Version> {
    let mut name = location.file_name()?.to_str()?;
    if !name.starts_with("python") {
        return None;
    }
    name = &name[6..];

    // Special case: If the executable name is exactly "python", this is an
    // valid Python interpreter with an unspecified version.
    if name.is_empty() {
        Some((-1, -1, -1))
    } else {
        parse_version_from_name(name, VersionPad::Unknown)
    }
}

#[derive(Eq)]
pub struct Python {
    location: String,
    version: Version,
    order: usize,   // Tie breaker (e.g. order in PATH).
}

impl Python {
    pub fn from_managed(root: path::PathBuf, order: usize) -> Option<Python> {
        let version = parse_managed_root_path(&root)?;
        let p = root.join("bin/python");
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


#[cfg(test)]
mod tests {
    use super::*;
    use super::VersionPad::{Zero, Unknown};
    use std::path::Path;
    use specs::Spec::{Major, Minor};

    #[test]
    fn test_parse_version_from_name() {
        assert_eq!(parse_version_from_name("3.6.3", Zero), Some((3, 6, 3)));

        // Pad default values.
        assert_eq!(parse_version_from_name("2.6", Zero), Some((2, 6, 0)));
        assert_eq!(parse_version_from_name("3", Zero), Some((3, 0, 0)));

        // Default values can be -1.
        assert_eq!(parse_version_from_name("2.6", Unknown), Some((2, 6, -1)));
        assert_eq!(parse_version_from_name("3", Unknown), Some((3, -1, -1)));
    }

    #[test]
    fn test_parse_version_from_name_trailing_garbage() {
        assert_eq!(parse_version_from_name("2.6-config", Zero), None);
    }

    // This might be supported in the future.
    #[test]
    fn test_parse_version_from_name_dev() {
        assert_eq!(parse_version_from_name("3.8-dev", Zero), None);
    }

    #[test]
    fn test_parse_managed_root_path() {
        assert_eq!(parse_managed_root_path(&Path::new("foo/3.6.1")),
                   Some((3, 6, 1)));
        assert_eq!(parse_managed_root_path(&Path::new("foo/CPython-3.6.1")),
                   Some((3, 6, 1)));
    }

    #[test]
    fn test_parse_managed_root_path_invalid() {
        assert_eq!(parse_managed_root_path(&Path::new(".DS_Store")), None);
        assert_eq!(parse_managed_root_path(&Path::new("foo/bar/.1")), None);
    }

    // This might be supported in the future.
    #[test]
    fn test_parse_managed_root_path_dev() {
        assert_eq!(parse_managed_root_path(&Path::new("foo/3.8-dev")), None);
    }

    // This might be supported in the future.
    #[test]
    fn test_parse_managed_root_path_alternative_implementation() {
        assert_eq!(parse_managed_root_path(&Path::new("PyPy-2.6.1")), None);
        assert_eq!(parse_managed_root_path(&Path::new("pypy3.5-6.0.0")), None);
    }

    #[test]
    fn test_parse_executable_path() {
        assert_eq!(parse_executable_path(&Path::new("python3.6")),
                   Some((3, 6, -1)));
        assert_eq!(parse_executable_path(&Path::new("python2")),
                   Some((2, -1, -1)));
        assert_eq!(parse_executable_path(&Path::new("python")),
                   Some((-1, -1, -1)));
    }

    #[test]
    fn test_parse_executable_path_invalid() {
        assert_eq!(parse_executable_path(&Path::new("2.7")), None);
        assert_eq!(parse_executable_path(&Path::new("python2.7-config")),
                   None);
    }

    // This might be supported in the future.
    #[test]
    fn test_parse_executable_path_m() {
        assert_eq!(parse_executable_path(&Path::new("python3.7m")), None);
    }

    #[test]
    fn test_python_matches() {
        let python = Python {
            location: String::new(),
            version: (1, 2, 3),
            order: 0,
        };
        assert!(python.matches(&Major(1)));
        assert!(python.matches(&Minor(1, 2)));
        assert!(!python.matches(&Major(2)));
        assert!(!python.matches(&Minor(2, 2)));
    }

    #[test]
    fn test_python_cmp() {
        let python370 = Python {
            location: String::new(),
            version: (3, 7, 0),
            order: 0,
        };
        let python371 = Python {
            location: String::new(),
            version: (3, 7, 1),
            order: 0,
        };
        assert!(python370 < python371);
    }
}
