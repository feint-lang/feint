use crate::util::Location;
use crate::vm::Code;

pub type CompResult = Result<Code, CompErr>;

#[derive(Clone, Debug)]
pub struct CompErr {
    pub kind: CompErrKind,
}

impl CompErr {
    fn new(kind: CompErrKind) -> Self {
        Self { kind }
    }

    pub fn label_not_found_in_scope(name: String) -> Self {
        Self::new(CompErrKind::LabelNotFoundInScope(name))
    }

    pub fn cannot_jump_out_of_func(name: String) -> Self {
        Self::new(CompErrKind::CannotJumpOutOfFunc(name))
    }

    pub fn duplicate_label_in_scope(
        name: String,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(CompErrKind::DuplicateLabelInScope(name, start, end))
    }

    pub fn expected_ident(start: Location, end: Location) -> Self {
        Self::new(CompErrKind::ExpectedIdent(start, end))
    }

    pub fn cannot_assign_special_ident(name: String) -> Self {
        Self::new(CompErrKind::CannotAssignSpecialIdent(name))
    }

    pub fn global_not_found<S: Into<String>>(
        name: S,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(CompErrKind::GlobalNotFound(name.into(), start, end))
    }

    pub fn var_args_must_be_last() -> Self {
        Self::new(CompErrKind::VarArgsMustBeLast)
    }

    pub fn loc(&self) -> (Location, Location) {
        use CompErrKind::*;
        let default = Location::default();
        match &self.kind {
            LabelNotFoundInScope(..) => (default, default),
            CannotJumpOutOfFunc(..) => (default, default),
            DuplicateLabelInScope(_, start, end) => (*start, *end),
            ExpectedIdent(start, end) => (*start, *end),
            CannotAssignSpecialIdent(..) => (default, default),
            GlobalNotFound(_, start, end) => (*start, *end),
            VarArgsMustBeLast => (default, default),
        }
    }
}

// TODO: Add start and end locations to all error types
#[derive(Clone, Debug)]
pub enum CompErrKind {
    LabelNotFoundInScope(String),
    CannotJumpOutOfFunc(String),
    DuplicateLabelInScope(String, Location, Location),
    ExpectedIdent(Location, Location),
    CannotAssignSpecialIdent(String),
    GlobalNotFound(String, Location, Location),
    VarArgsMustBeLast,
}
