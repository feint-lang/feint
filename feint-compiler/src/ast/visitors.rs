//! Miscellaneous AST visitors.

use super::ast;
use std::collections::HashSet;

/// Find import statements in module AST.
///
/// XXX: Currently, this only looks at top level statements... but
///      perhaps import should only be allowed at the top level anyway?
pub struct ImportVisitor {
    imports: Vec<(String, Option<String>)>,
}

impl ImportVisitor {
    pub fn new() -> Self {
        Self { imports: vec![] }
    }

    pub fn imports(&self) -> &Vec<(String, Option<String>)> {
        &self.imports
    }

    pub fn visit_module(&mut self, node: &ast::Module) {
        self.visit_statements(&node.statements)
    }

    fn visit_statements(&mut self, statements: &[ast::Statement]) {
        statements.iter().for_each(|s| self.visit_statement(s));
    }

    fn visit_statement(&mut self, statement: &ast::Statement) {
        if let ast::StatementKind::Import(name, as_name) = &statement.kind {
            if !self.imports.iter().any(|(n, _)| n == name) {
                self.imports.push((name.to_owned(), as_name.to_owned()));
            }
        }
    }
}

/// Find globals in module AST. At this stage, we can extract the names
/// of the globals but not their values.
pub struct GlobalsNamesVisitor {
    global_names: HashSet<String>,
}

impl Default for GlobalsNamesVisitor {
    fn default() -> Self {
        GlobalsNamesVisitor::new(HashSet::default())
    }
}

#[allow(dead_code)]
impl GlobalsNamesVisitor {
    pub fn new(global_names: HashSet<String>) -> Self {
        Self { global_names }
    }

    pub fn with_module_globals() -> Self {
        let global_names = ["$full_name", "$name", "$path", "$doc"]
            .into_iter()
            .map(|n| n.to_owned())
            .collect();
        GlobalsNamesVisitor::new(global_names)
    }

    pub fn global_names(&self) -> &HashSet<String> {
        &self.global_names
    }

    pub fn take_global_names(self) -> HashSet<String> {
        self.global_names
    }

    pub fn visit_module(&mut self, node: &ast::Module) {
        self.visit_statements(&node.statements)
    }

    fn visit_statements(&mut self, statements: &[ast::Statement]) {
        statements.iter().for_each(|s| self.visit_statement(s));
    }

    fn visit_statement(&mut self, statement: &ast::Statement) {
        if let ast::StatementKind::Import(name, as_name) = &statement.kind {
            let declared_name = if let Some(as_name) = as_name {
                as_name
            } else {
                name.split('.').last().unwrap()
            };
            self.global_names.insert(declared_name.to_owned());
        } else if let Some(expr) = statement.expr() {
            if let Some((left, _right)) = expr.assignment() {
                if let Some(name) = left.ident_name() {
                    self.global_names.insert(name);
                }
            }
        }
    }
}
