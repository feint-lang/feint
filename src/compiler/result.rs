use crate::source::Location;
use crate::types::Module;

pub type CompResult = Result<Module, CompErr>;

#[derive(Clone, Debug)]
pub struct CompErr {
    pub kind: CompErrKind,
}

impl CompErr {
    fn new(kind: CompErrKind) -> Self {
        Self { kind }
    }
    pub fn name_not_found(name: String, start: Location, end: Location) -> Self {
        Self::new(CompErrKind::NameNotFound(name, start, end))
    }

    pub fn label_not_found_in_scope(
        name: String,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(CompErrKind::LabelNotFoundInScope(name, start, end))
    }

    pub fn cannot_jump_out_of_func(
        name: String,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(CompErrKind::CannotJumpOutOfFunc(name, start, end))
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

    pub fn cannot_assign_special_ident(
        name: String,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(CompErrKind::CannotAssignSpecialIdent(name, start, end))
    }

    pub fn main_must_be_func(start: Location, end: Location) -> Self {
        Self::new(CompErrKind::MainMustBeFunc(start, end))
    }

    pub fn global_not_found<S: Into<String>>(
        name: S,
        start: Location,
        end: Location,
    ) -> Self {
        Self::new(CompErrKind::GlobalNotFound(name.into(), start, end))
    }

    pub fn var_args_must_be_last(start: Location, end: Location) -> Self {
        Self::new(CompErrKind::VarArgsMustBeLast(start, end))
    }

    pub fn print<S: Into<String>>(msg: S, start: Location, end: Location) -> Self {
        Self::new(CompErrKind::Print(msg.into(), start, end))
    }

    pub fn loc(&self) -> (Location, Location) {
        use CompErrKind::*;
        let (start, end) = match &self.kind {
            NameNotFound(_, start, end) => (start, end),
            LabelNotFoundInScope(_, start, end) => (start, end),
            CannotJumpOutOfFunc(_, start, end) => (start, end),
            DuplicateLabelInScope(_, start, end) => (start, end),
            ExpectedIdent(start, end) => (start, end),
            CannotAssignSpecialIdent(_, start, end) => (start, end),
            MainMustBeFunc(start, end) => (start, end),
            GlobalNotFound(_, start, end) => (start, end),
            VarArgsMustBeLast(start, end) => (start, end),
            Print(_, start, end) => (start, end),
        };
        (*start, *end)
    }
}

// TODO: Add start and end locations to all error types
#[derive(Clone, Debug)]
pub enum CompErrKind {
    NameNotFound(String, Location, Location),
    LabelNotFoundInScope(String, Location, Location),
    CannotJumpOutOfFunc(String, Location, Location),
    DuplicateLabelInScope(String, Location, Location),
    ExpectedIdent(Location, Location),
    CannotAssignSpecialIdent(String, Location, Location),
    MainMustBeFunc(Location, Location),
    GlobalNotFound(String, Location, Location),
    VarArgsMustBeLast(Location, Location),
    Print(String, Location, Location),
}
