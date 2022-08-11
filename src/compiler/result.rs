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

    pub fn unhandled_expr(start: Location, end: Location) -> Self {
        Self::new(CompErrKind::UnhandledExpr(start, end))
    }

    pub fn label_not_found_in_scope(name: String) -> Self {
        Self::new(CompErrKind::LabelNotFoundInScope(name))
    }

    pub fn cannot_jump_out_of_func(name: String) -> Self {
        Self::new(CompErrKind::CannotJumpOutOfFunc(name))
    }

    pub fn duplicate_label_in_scope(name: String) -> Self {
        Self::new(CompErrKind::DuplicateLabelInScope(name))
    }

    pub fn expected_ident() -> Self {
        Self::new(CompErrKind::ExpectedIdent)
    }

    pub fn cannot_assign_special_ident(name: String) -> Self {
        Self::new(CompErrKind::CannotAssignSpecialIdent(name))
    }

    pub fn var_args_must_be_last() -> Self {
        Self::new(CompErrKind::VarArgsMustBeLast)
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
    VarArgsMustBeLast,
}
