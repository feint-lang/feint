use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use once_cell::sync::Lazy;

use crate::vm::{RuntimeErr, DEFAULT_MAX_CALL_DEPTH};

pub static CONFIG: Lazy<Arc<RwLock<Config>>> =
    Lazy::new(|| Arc::new(RwLock::new(Config::default())));

pub struct Config {
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
        use ValKind::*;
        let mut entries = HashMap::new();
        entries.insert(
            "builtin_module_search_path".to_owned(),
            Str("src/modules".to_owned()),
        );
        entries.insert("max_call_depth".to_owned(), Usize(DEFAULT_MAX_CALL_DEPTH));
        entries.insert("debug".to_owned(), Bool(false));
        Self { entries }
    }
}

impl Config {
    fn check_name(&self, name: &str) -> NameResult {
        if self.entries.contains_key(name) {
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
