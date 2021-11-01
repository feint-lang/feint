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
    // XXX: Not sure this belongs here. Move to its own module?
    pub fn format(&self, vm: &mut VM) -> Result<Self, RuntimeErr> {
        assert!(self.is_format_string, "String is not a format string: {}", self);

        let value = self.value();
        let mut formatted = RustString::new();
        let mut chars = value.chars();
        let mut peek_chars = self.value.chars();

        // Current group expression
        let mut expr = RustString::with_capacity(32);

        let mut pos = 0;
        let mut stack: Vec<usize> = Vec::new();

        peek_chars.next();

        while let Some(c) = chars.next() {
            let d = peek_chars.next();

            if (c, d) == ('\\', Some('$')) {
                chars.next();
                peek_chars.next();
            } else if (c, d) == ('$', Some('{')) {
                chars.next();
                peek_chars.next();
                stack.push(pos);
                pos += 1;

                while let Some(c) = chars.next() {
                    peek_chars.next();
                    pos += 1;

                    if c == '}' {
                        if let Some(start) = stack.pop() {
                            expr.push_str(&value[start + 2..pos]);
                            println!("EXPR: `{}` @ {}:{}", expr, start, pos);
                        }

                        let trimmed_expr = expr.trim();
                        if trimmed_expr.len() == 0 {
                            return Err(RuntimeErr::new(RuntimeErrKind::SyntaxError(
                                format!(
                                    "Empty expression in $ string at position {}",
                                    pos,
                                ),
                            )));
                        }

                        let result = execute_text(vm, expr.trim(), false, false);

                        expr.clear();

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

                        break;
                    }
                }
            } else {
                formatted.push(c);
            }

            pos += 1;
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
