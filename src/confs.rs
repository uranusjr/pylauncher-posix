extern crate ini;

use std::env;
use std::path;

fn get_ini(dir: &path::Path) -> Option<ini::Ini> {
    let p = dir.join("py.ini");
    if p.is_file() {
        dbg!("Reading INI" => &p);
        ini::Ini::load_from_file(p).ok()
    } else {
        dbg!("INI not found" => &p);
        None
    }
}

pub struct Conf {
    src: Option<ini::Ini>,
}

impl Conf {
    pub fn new() -> Self {
        let src = env::var("HOME").ok().and_then(|ref home| {
            get_ini(&path::Path::new(home).join(".local/share"))
        });
        Self { src: src }
    }

    fn value(&self, sec: &str, key: &str) -> Option<String> {
        self.src.as_ref()?.get_from(Some(sec), key).map(|s| s.to_owned())
    }

    fn default_value(&self, key: &str) -> Option<String> {
        // TODO: Both the environment variable and INI keys are
        // case-insensitive on Windows. Does it make sense to do this here?
        env::var(format!("PY_{}", key.to_ascii_uppercase())).ok().or_else(|| {
            let value = self.value("defaults", &key.to_ascii_lowercase());
            dbg!("Key" => &key, "Value" => &value);
            value
        })
    }

    pub fn default_python(&self) -> Option<String> {
        self.default_value("python")
    }

    pub fn default_python_for(&self, major: u8) -> Option<String> {
        self.default_value(&format!("python{}", major))
    }
}
