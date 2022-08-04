use crate::util::Location;
use crate::vm::Code;

pub type CompResult = Result<Code, CompErr>;

#[derive(Clone, Debug)]
pub struct CompErr {
    pub kind: CompErrKind,
}

impl CompErr {
    pub fn new(kind: CompErrKind) -> Self {
        Self { kind }
    }

    pub fn new_unhandled_expr(start: Location, end: Location) -> Self {
        Self { kind: CompErrKind::UnhandledExpr(start, end) }
    }

    pub fn new_label_not_found_in_scope(name: String) -> Self {
        Self { kind: CompErrKind::LabelNotFoundInScope(name) }
    }

    pub fn new_cannot_jump_out_of_func(name: String) -> Self {
        Self { kind: CompErrKind::CannotJumpOutOfFunc(name) }
    }

    pub fn new_duplicate_label_in_scope(name: String) -> Self {
        Self { kind: CompErrKind::DuplicateLabelInScope(name) }
    }

    pub fn new_expected_ident() -> Self {
        Self { kind: CompErrKind::ExpectedIdent }
    }

    pub fn new_cannot_assign_special_ident(name: String) -> Self {
        Self { kind: CompErrKind::CannotAssignSpecialIdent(name) }
    }
}

#[derive(Clone, Debug)]
pub enum CompErrKind {
    UnhandledExpr(Location, Location),
    LabelNotFoundInScope(String),
    CannotJumpOutOfFunc(String),
    DuplicateLabelInScope(String),
    ExpectedIdent,
    CannotAssignSpecialIdent(String),
}
