//! Miscellaneous AST visitors.

use super::ast;

/// Find import statements in module AST.
///
/// XXX: Currently, this only looks at top level statements... but
///      perhaps import should only be allowed at the top level anyway?
pub struct ImportVisitor {
    imports: Vec<String>,
}

impl ImportVisitor {
    pub fn new() -> Self {
        Self { imports: vec![] }
    }

    pub fn imports(&self) -> &Vec<String> {
        &self.imports
    }

    pub fn visit_module(&mut self, node: &ast::Module) {
        self.visit_statements(&node.statements)
    }

    fn visit_statements(&mut self, statements: &[ast::Statement]) {
        statements.iter().for_each(|s| self.visit_statement(s));
    }

    fn visit_statement(&mut self, statement: &ast::Statement) {
        if let ast::StatementKind::Import(name) = &statement.kind {
            if !self.imports.contains(name) {
                self.imports.push(name.to_owned());
            }
        }
    }
}
