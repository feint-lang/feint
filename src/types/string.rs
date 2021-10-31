//! Built in string type
use std::any::Any;
use std::fmt;

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
    pub fn format(&self, vm: &mut VM) -> Result<Self, RuntimeErr> {
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
                    let mut expr = RustString::new();
                    loop {
                        if let Some(c) = chars.next() {
                            peek_chars.next();
                            if c == '}' {
                                // Execute expression then pop result
                                // from stack.
                                // XXX: This feels a little wonky?
                                match execute_text(vm, expr.trim(), false, false) {
                                    Ok(_) => (),
                                    Err(err) => return Err(err),
                                }
                                if let Some(i) = vm.pop() {
                                    if let Some(obj) = vm.ctx.get_obj(i) {
                                        formatted.push_str(obj.to_string().as_str());
                                    } else {
                                        let err = RuntimeErrKind::ObjectNotFound(i);
                                        return Err(RuntimeErr::new(err));
                                    }
                                } else {
                                    let err = RuntimeErrKind::EmptyStack;
                                    return Err(RuntimeErr::new(err));
                                }
                                break;
                            } else {
                                expr.push(c);
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
