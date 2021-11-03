//! Built in string type
use std::any::Any;
use std::fmt;

use crate::format::{scan, Token as FStringToken};

use crate::vm::{
    execute_text, ExeResult, RuntimeBoolResult, RuntimeContext, RuntimeErr,
    RuntimeErrKind, RuntimeResult, VM,
};

use crate::types::class::TypeRef;
use crate::types::object::{Object, ObjectExt, ObjectRef};

type RustString = std::string::String;

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

    // TODO: Handle nested ${}
    // XXX: Not sure this belongs here. Move to its own module?
    // XXX: Maybe all the scanning/parsing should happen in the main
    //      scanner? In addition to catching syntax errors early, I
    //      think this would make it easier to handle nested groups.
    pub fn format(&self, vm: &mut VM) -> Result<Self, RuntimeErr> {
        assert!(self.is_format_string, "String is not a format string: {}", self);

        let value = self.value();
        let result = scan(value);
        let tokens = result.expect("Scanning of format string failed");
        let mut formatted = RustString::with_capacity(64);

        for token in tokens {
            match token {
                FStringToken::String(part, _) => {
                    formatted.push_str(part.as_str());
                }
                FStringToken::Group(expr, _) => {
                    let result = execute_text(vm, expr.as_str(), false, false);
                    if result.is_err() {
                        return Err(result.unwrap_err());
                    }
                    if let Some(i) = vm.pop() {
                        if let Some(obj) = vm.ctx.get_obj(i) {
                            formatted.push_str(obj.to_string().as_str());
                        } else {
                            return Err(RuntimeErr::new(
                                RuntimeErrKind::ObjectNotFound(i),
                            ));
                        }
                    } else {
                        return Err(RuntimeErr::new(RuntimeErrKind::EmptyStack));
                    }
                }
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
            Err(RuntimeErr::new_type_error(format!(
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
            Err(RuntimeErr::new_type_error(format!(
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

impl fmt::Debug for String {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = if self.is_format_string { "$" } else { "" };
        write!(f, "{}\"{}\"", prefix, self.value())
    }
}
