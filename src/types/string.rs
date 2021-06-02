//! Built in string type
use std::any::Any;
use std::fmt;

use crate::vm::{
    RuntimeBoolResult, RuntimeContext, RuntimeError, RuntimeErrorKind, RuntimeResult,
};

use crate::types::class::TypeRef;
use crate::types::object::{Object, ObjectExt, ObjectRef};

type RustString = std::string::String;

#[derive(Debug)]
pub struct String {
    class: TypeRef,
    value: RustString,
    is_format_string: bool, // is this a format string?
}

impl String {
    pub fn new<S: Into<RustString>>(class: TypeRef, value: S, format: bool) -> Self {
        Self { class, value: value.into(), is_format_string: format }
    }

    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    pub fn is_format_string(&self) -> bool {
        self.is_format_string
    }

    pub fn format(&self, ctx: &RuntimeContext) -> Result<Self, RuntimeError> {
        assert!(self.is_format_string, "String is not a format string: {}", self);
        let mut formatted = RustString::new();
        let mut chars = self.value().chars();
        let mut peek_chars = self.value.chars();
        peek_chars.next();
        loop {
            if let Some(c) = chars.next() {
                let d = peek_chars.next();
                if let ('$', Some('{')) = (c, d) {
                    chars.next();
                    peek_chars.next();
                    let mut name = RustString::new();
                    loop {
                        if let Some(c) = chars.next() {
                            peek_chars.next();
                            if c == '}' {
                                if let Some(obj) = ctx.get_obj_by_name(name.as_str()) {
                                    formatted.push_str(obj.to_string().as_str());
                                } else {
                                    return Err(RuntimeError::new(
                                        RuntimeErrorKind::NameError(format!(
                                            "Name not found: {}",
                                            name
                                        )),
                                    ));
                                }
                                break;
                            } else {
                                name.push(c);
                            }
                        } else {
                            break;
                        }
                    }
                } else {
                    formatted.push(c);
                }
            } else {
                break;
            }
        }
        Ok(Self::new(self.class().clone(), formatted, false))
    }
}

impl Object for String {
    fn class(&self) -> &TypeRef {
        &self.class
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn is_equal(&self, rhs: ObjectRef, _ctx: &RuntimeContext) -> RuntimeBoolResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            Ok(self.is(rhs) || self.value() == rhs.value())
        } else {
            Err(RuntimeError::new_type_error(format!(
                "Could not compare String to {} for equality",
                rhs.class().name()
            )))
        }
    }

    fn add(&self, rhs: ObjectRef, ctx: &RuntimeContext) -> RuntimeResult {
        if let Some(rhs) = rhs.as_any().downcast_ref::<Self>() {
            let a = self.value();
            let b = rhs.value();
            let mut value = RustString::with_capacity(a.len() + b.len());
            value.push_str(a);
            value.push_str(b);
            let value = ctx.builtins.new_string(value, false);
            Ok(value)
        } else {
            Err(RuntimeError::new_type_error(format!(
                "Could not concatenate String with {}",
                rhs.class().name()
            )))
        }
    }
}

// Display -------------------------------------------------------------

impl fmt::Display for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value())
    }
}
