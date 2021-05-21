pub struct TypeError {
    kind: TypeErrorKind,
}

impl TypeError {
    pub fn new(kind: TypeErrorKind) -> Self {
        Self { kind }
    }
}

pub enum TypeErrorKind {
    TypeError,
}

pub struct ObjectError {
    kind: ObjectErrorKind,
}

impl ObjectError {
    pub fn new(kind: ObjectErrorKind) -> Self {
        Self { kind }
    }
}

pub enum ObjectErrorKind {
    AttributeDoesNotExist(String),
    AttributeCannotBeSet(String),
}
