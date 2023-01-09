use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::RuntimeErr;

pub static CONFIG: Lazy<Arc<RwLock<Config>>> =
    Lazy::new(|| Arc::new(RwLock::new(Config::default())));

pub struct Config {
    known_names: HashSet<String>,
    entries: HashMap<String, ValKind>,
}

pub enum ValKind {
    Bool(bool),
    Str(String),
    Usize(usize),
}

type NameResult = Result<(), RuntimeErr>;

impl Default for Config {
    fn default() -> Self {
        let names = [
            "builtin_module_search_path".to_owned(),
            "max_call_depth".to_owned(),
            "debug".to_owned(),
        ];
        let names = HashSet::from(names);
        Self::new(names)
    }
}

impl Config {
    pub fn new(known_names: HashSet<String>) -> Self {
        Self { known_names, entries: HashMap::new() }
    }

    fn check_name(&self, name: &str) -> NameResult {
        if self.known_names.contains(name) {
            Ok(())
        } else {
            Err(RuntimeErr::config_name_not_known(name))
        }
    }

    fn get(&self, name: &str) -> Result<&ValKind, RuntimeErr> {
        self.check_name(name)?;
        if let Some(val) = self.entries.get(name) {
            Ok(val)
        } else {
            Err(RuntimeErr::config_value_not_set(name))
        }
    }

    pub fn get_bool(&self, name: &str) -> Result<bool, RuntimeErr> {
        let val = self.get(name)?;
        if let ValKind::Bool(val) = val {
            Ok(*val)
        } else {
            Err(RuntimeErr::config_value_is_not_valid(name, "expected bool"))
        }
    }

    pub fn get_str(&self, name: &str) -> Result<&String, RuntimeErr> {
        let val = self.get(name)?;
        if let ValKind::Str(val) = val {
            Ok(val)
        } else {
            Err(RuntimeErr::config_value_is_not_valid(name, "expected string"))
        }
    }

    pub fn get_usize(&self, name: &str) -> Result<usize, RuntimeErr> {
        let val = self.get(name)?;
        if let ValKind::Usize(val) = val {
            Ok(*val)
        } else {
            Err(RuntimeErr::config_value_is_not_valid(name, "expected usize"))
        }
    }

    fn set(&mut self, name: &str, val: ValKind) -> NameResult {
        self.check_name(name)?;
        self.entries.insert(name.to_owned(), val);
        Ok(())
    }

    pub fn set_bool(&mut self, name: &str, val: bool) -> NameResult {
        self.set(name, ValKind::Bool(val))
    }

    pub fn set_str(&mut self, name: &str, val: &str) -> NameResult {
        self.set(name, ValKind::Str(val.to_owned()))
    }

    pub fn set_usize(&mut self, name: &str, val: usize) -> NameResult {
        self.set(name, ValKind::Usize(val))
    }
}
