use num_traits::ToPrimitive;

use crate::ast;
use crate::types::{create, ObjectRef};
use crate::util::{
    BinaryOperator, CompareOperator, InplaceOperator, UnaryCompareOperator,
    UnaryOperator,
};
use crate::vm::{Code, Inst};

use super::result::{CompErr, CompResult};
use super::scope::{Scope, ScopeKind, ScopeTree};

// Compiler ------------------------------------------------------------

/// Compile AST to code object.
pub fn compile(program: ast::Program, argv: Vec<&str>) -> CompResult {
    let argv = argv.into_iter().map(|a| a.to_owned()).collect();
    let mut visitor = Visitor::new(argv);
    visitor.visit_program(program)?;
    Ok(visitor.code)
}

// Visitor -------------------------------------------------------------

type VisitResult = Result<(), CompErr>;

struct Visitor {
    code: Code,
    scope_tree: ScopeTree,
    scope_depth: usize,
    has_main: bool,
    argv: Vec<String>,
}

impl Visitor {
    fn new(argv: Vec<String>) -> Self {
        Self {
            code: Code::new(),
            scope_tree: ScopeTree::new(),
            scope_depth: 0,
            has_main: false,
            argv,
        }
    }

    // Visitors --------------------------------------------------------

    fn visit_program(&mut self, node: ast::Program) -> VisitResult {
        self.visit_statements(node.statements)?;
        assert_eq!(self.scope_tree.pointer(), 0);
        self.fix_jumps()?;
        if self.has_main {
            let argv: Vec<ObjectRef> = self.argv.iter().map(create::new_str).collect();
            let argc = argv.len();
            for arg in argv {
                self.add_const(arg);
            }
            self.push(Inst::LoadVar("$main".to_string()));
            self.push(Inst::Call(argc));
            self.push(Inst::Return);
            self.push(Inst::HaltTop);
        } else {
            self.push(Inst::Halt(0));
        }
        Ok(())
    }

    fn visit_statements(&mut self, statements: Vec<ast::Statement>) -> VisitResult {
        for statement in statements {
            self.visit_statement(statement)?;
        }
        Ok(())
    }

    fn visit_statement(&mut self, node: ast::Statement) -> VisitResult {
        type Kind = ast::StatementKind;
        match node.kind {
            Kind::Jump(name) => {
                let jump_addr = self.len();
                self.push(Inst::Placeholder(
                    0,
                    Box::new(Inst::Jump(0, 0)),
                    "Jump address not set to label address".to_owned(),
                ));
                self.scope_tree.add_jump(name.as_str(), jump_addr);
            }
            Kind::Label(name, expr) => {
                let addr = self.len();
                self.visit_expr(expr, None)?;
                if self.scope_tree.add_label(name.as_str(), addr).is_some() {
                    return Err(CompErr::new_duplicate_label_in_scope(name));
                }
            }
            Kind::Break(expr) => self.visit_break(expr)?,
            Kind::Continue => self.visit_continue()?,
            Kind::Expr(expr) => self.visit_expr(expr, None)?,
        }
        if self.scope_tree.in_global_scope() {
            self.push(Inst::Pop);
        }
        Ok(())
    }

    fn visit_break(&mut self, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr, None)?;
        self.push(Inst::BreakPlaceholder(self.len(), self.scope_depth));
        Ok(())
    }

    fn visit_continue(&mut self) -> VisitResult {
        self.push(Inst::ContinuePlaceholder(self.len(), self.scope_depth));
        Ok(())
    }

    fn visit_exprs(&mut self, exprs: Vec<ast::Expr>) -> VisitResult {
        for expr in exprs {
            self.visit_expr(expr, None)?;
        }
        Ok(())
    }

    /// Visit an expression. The `name` argument is currently only
    /// used to assign names to functions.
    fn visit_expr(&mut self, node: ast::Expr, name: Option<String>) -> VisitResult {
        type Kind = ast::ExprKind;
        match node.kind {
            Kind::Tuple(items) => self.visit_tuple(items)?,
            Kind::Literal(literal) => self.visit_literal(literal)?,
            Kind::FormatString(items) => self.visit_format_string(items)?,
            Kind::Ident(ident) => self.visit_ident(ident)?,
            Kind::Assignment(ident, expr) => self.visit_assignment(ident, *expr)?,
            Kind::Block(block) => self.visit_block(block)?,
            Kind::Conditional(branches, default) => {
                self.visit_conditional(branches, default)?
            }
            Kind::Loop(expr, block) => self.visit_loop(*expr, block)?,
            Kind::Func(func) => self.visit_func(func, name)?,
            Kind::Call(call) => self.visit_call(call)?,
            Kind::UnaryOp(op, b) => self.visit_unary_op(op, *b)?,
            Kind::UnaryCompareOp(op, b) => self.visit_unary_compare_op(op, *b)?,
            Kind::BinaryOp(a, op, b) => self.visit_binary_op(*a, op, *b)?,
            Kind::CompareOp(a, op, b) => self.visit_compare_op(*a, op, *b)?,
            Kind::InplaceOp(a, op, b) => self.visit_inplace_op(*a, op, *b)?,
        }
        Ok(())
    }

    fn visit_tuple(&mut self, items: Vec<ast::Expr>) -> VisitResult {
        let num_items = items.len();
        self.visit_exprs(items)?;
        self.push(Inst::MakeTuple(num_items));
        Ok(())
    }

    fn visit_literal(&mut self, node: ast::Literal) -> VisitResult {
        type Kind = ast::LiteralKind;
        match node.kind {
            Kind::Nil => self.push_nil(),
            Kind::Bool(true) => self.push_true(),
            Kind::Bool(false) => self.push_false(),
            Kind::Ellipsis => self.push_nil(),
            Kind::Int(value) => {
                if create::in_shared_int_range(&value) {
                    let index = value.to_usize().unwrap() + 3;
                    self.push(Inst::LoadGlobalConst(index))
                } else {
                    self.add_const(create::new_int(value));
                }
            }
            Kind::Float(value) => {
                self.add_const(create::new_float(value));
            }
            Kind::String(value) => {
                self.add_const(create::new_str(value));
            }
        }
        Ok(())
    }

    fn visit_format_string(&mut self, items: Vec<ast::Expr>) -> VisitResult {
        let num_items = items.len();
        self.visit_exprs(items)?;
        self.push(Inst::MakeString(num_items));
        Ok(())
    }

    /// Visit identifier as expression (i.e., not as part of an
    /// assignment).
    fn visit_ident(&mut self, node: ast::Ident) -> VisitResult {
        type Kind = ast::IdentKind;
        match node.kind {
            Kind::Ident(name) => self.push(Inst::LoadVar(name)),
            Kind::SpecialIdent(name) => self.push(Inst::LoadVar(name)),
            Kind::TypeIdent(name) => self.push(Inst::LoadVar(name)),
        }
        Ok(())
    }

    fn visit_get_attr(
        &mut self,
        obj_expr: ast::Expr,
        name_expr: ast::Expr,
    ) -> VisitResult {
        self.visit_expr(obj_expr, None)?;
        if let Some(name) = name_expr.is_ident() {
            self.visit_literal(ast::Literal::new_string(name))?;
        } else if let Some(name) = name_expr.is_special_ident() {
            self.visit_literal(ast::Literal::new_string(name))?;
        } else if let Some(name) = name_expr.is_type_ident() {
            self.visit_literal(ast::Literal::new_string(name))?;
        } else {
            self.visit_expr(name_expr, None)?;
        }
        self.push(Inst::BinaryOp(BinaryOperator::Dot));
        Ok(())
    }

    fn visit_block(&mut self, node: ast::StatementBlock) -> VisitResult {
        self.push(Inst::ScopeStart);
        self.enter_scope(ScopeKind::Block);
        self.visit_statements(node.statements)?;
        self.push(Inst::ScopeEnd);
        self.exit_scope();
        Ok(())
    }

    fn visit_conditional(
        &mut self,
        branches: Vec<(ast::Expr, ast::StatementBlock)>,
        default: Option<ast::StatementBlock>,
    ) -> VisitResult {
        assert!(!branches.is_empty(), "At least one branch required for conditional");

        // Addresses of branch jump-out instructions (added after each
        // branch's block). The target address for these isn't known
        // until the whole conditional suite is compiled.
        let mut jump_out_addrs: Vec<usize> = vec![];

        for (expr, block) in branches {
            // Evaluate branch expression.
            self.visit_expr(expr, None)?;

            // Placeholder for jump depending on result of branch expr.
            let jump_index = self.len();
            self.push(Inst::Placeholder(
                jump_index,
                Box::new(Inst::JumpIfElse(0, 0, 0)),
                "Branch condition jump not set".to_owned(),
            ));

            // Start of branch block (jump target if branch condition is
            // true).
            let block_addr = jump_index + 1;
            self.visit_block(block)?;

            // Placeholder for jump out of conditional suite if this
            // branch is selected.
            let jump_out_addr = self.len();
            jump_out_addrs.push(jump_out_addr);
            self.push(Inst::Placeholder(
                jump_out_addr,
                Box::new(Inst::Jump(0, 0)),
                "Branch jump out not set".to_owned(),
            ));

            // Jump target if branch condition is false.
            let next_addr = self.len();

            self.code
                .replace_inst(jump_index, Inst::JumpIfElse(block_addr, next_addr, 0));
        }

        // Default block (if present).
        if let Some(default_block) = default {
            self.visit_block(default_block)?;
        } else {
            self.push_nil();
        }

        // Address of instruction after conditional suite.
        let after_addr = self.len();

        // Replace jump-out placeholders with actual jumps.
        for addr in jump_out_addrs {
            self.code.replace_inst(addr, Inst::Jump(after_addr, 0));
        }

        Ok(())
    }

    fn visit_loop(
        &mut self,
        expr: ast::Expr,
        block: ast::StatementBlock,
    ) -> VisitResult {
        let loop_scope_depth = self.scope_depth;
        let loop_addr = self.len();
        let true_cond = expr.is_true();
        let jump_out_index = if true_cond {
            // Skip evaluation since we know it will always succeed.
            self.push(Inst::NoOp);
            0
        } else {
            // Evaluate loop expression on every iteration.
            self.visit_expr(expr, None)?;
            // Placeholder for jump-out if result is false.
            let jump_out_index = self.len();
            self.push(Inst::Placeholder(
                jump_out_index,
                Box::new(Inst::JumpIfNot(0, 0)),
                "Jump-out for loop not set".to_owned(),
            ));
            jump_out_index
        };
        // Run the loop body.
        self.visit_block(block)?;
        // Jump to top of loop.
        self.push(Inst::Jump(loop_addr, 0));
        // Jump-out address.
        let after_addr = self.len();
        // Set address of jump-out placeholder (not needed if loop
        // expression is always true).
        if !true_cond {
            self.code.replace_inst(jump_out_index, Inst::JumpIfNot(after_addr, 0));
        }
        // Set address of breaks and continues.
        for addr in loop_addr..after_addr {
            let inst = &self.code[addr];
            if let Inst::BreakPlaceholder(break_addr, depth) = inst {
                self.code.replace_inst(
                    *break_addr,
                    Inst::Jump(after_addr, depth - loop_scope_depth),
                );
            } else if let Inst::ContinuePlaceholder(continue_addr, depth) = inst {
                self.code.replace_inst(
                    *continue_addr,
                    Inst::JumpPushNil(loop_addr, depth - loop_scope_depth),
                );
            }
        }
        Ok(())
    }

    fn visit_func(&mut self, node: ast::Func, name: Option<String>) -> VisitResult {
        let mut visitor = Visitor::new(vec![]);
        let name = if let Some(name) = name {
            self.has_main = name == "$main" && self.scope_tree.in_global_scope();
            name
        } else {
            "<anonymous>".to_owned()
        };
        let params = node.params;
        let return_nil = if let Some(last) = node.block.statements.last() {
            !matches!(last.kind, ast::StatementKind::Expr(_))
        } else {
            unreachable!("Block for function contains no statements");
        };
        visitor.push(Inst::ScopeStart);
        visitor.enter_scope(ScopeKind::Func);
        visitor.visit_statements(node.block.statements)?;
        visitor.fix_jumps()?;
        if return_nil {
            visitor.push_nil();
        }
        visitor.push(Inst::Return);
        visitor.push(Inst::ScopeEnd);
        visitor.exit_scope();
        assert_eq!(visitor.scope_tree.pointer(), 0);
        let mut code = visitor.code;

        let locals = if params.is_some() {
            let num_inst = code.len_chunk();
            let mut names = params.clone().unwrap();
            let mut locals = vec![];
            let mut assigned = vec![];
            let mut ip: usize = 0;
            let mut scope_level = 0;
            loop {
                if let Inst::ScopeStart = &code[ip] {
                    scope_level += 1;
                } else if let Inst::ScopeEnd = &code[ip] {
                    scope_level -= 1;
                } else if let Inst::DeclareVar(name) = &code[ip] {
                    // Create a local cell for the var
                    if scope_level == 1 {
                        let pos = names.iter().position(|p| name == p);
                        if pos.is_none() {
                            names.push(name.clone());
                            locals.push(create::new_nil());
                        }
                    }
                } else if let Inst::AssignVar(name) = &code[ip] {
                    if scope_level == 1 {
                        let pos = names.iter().position(|p| name == p);
                        if let Some(index) = pos {
                            code.replace_inst(
                                ip,
                                Inst::AssignVarAndStoreLocal(name.clone(), index),
                            );
                        }
                    } else {
                        assigned.push((scope_level, name.clone()));
                    }
                } else if let Inst::LoadVar(name) = &code[ip] {
                    if scope_level == 1
                        || !assigned.iter().any(|(s, n)| s == &scope_level && n == name)
                    {
                        let pos = names.iter().position(|p| name == p);
                        if let Some(index) = pos {
                            code.replace_inst(ip, Inst::LoadLocal(index));
                        }
                    }
                }
                ip += 1;
                if ip == num_inst {
                    break;
                }
            }
            locals
        } else {
            vec![]
        };

        let func = create::new_func(name, params, code, locals);
        self.add_const(func);
        Ok(())
    }

    fn visit_call(&mut self, node: ast::Call) -> VisitResult {
        let callable = node.callable;
        let args = node.args;
        let num_args = args.len();
        self.visit_exprs(args)?;
        self.visit_expr(*callable, None)?;
        self.push(Inst::Call(num_args));
        Ok(())
    }

    fn visit_unary_op(&mut self, op: UnaryOperator, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr, None)?;
        self.push(Inst::UnaryOp(op));
        Ok(())
    }

    fn visit_unary_compare_op(
        &mut self,
        op: UnaryCompareOperator,
        expr: ast::Expr,
    ) -> VisitResult {
        self.visit_expr(expr, None)?;
        self.push(Inst::UnaryCompareOp(op));
        Ok(())
    }

    fn visit_binary_op(
        &mut self,
        expr_a: ast::Expr,
        op: BinaryOperator,
        expr_b: ast::Expr,
    ) -> VisitResult {
        use BinaryOperator::*;
        match op {
            Dot => self.visit_get_attr(expr_a, expr_b),
            _ => {
                self.visit_expr(expr_a, None)?;
                self.visit_expr(expr_b, None)?;
                self.push(Inst::BinaryOp(op));
                Ok(())
            }
        }
    }

    fn visit_assignment(
        &mut self,
        ident: ast::Ident,
        value_expr: ast::Expr,
    ) -> VisitResult {
        use ast::IdentKind::{Ident, SpecialIdent};
        let name = match ident.kind {
            Ident(name) => {
                if name == "this" {
                    return Err(CompErr::new_cannot_assign_special_ident(name));
                }
                name
            }
            SpecialIdent(name) => {
                if name == "$main" && self.scope_tree.in_global_scope() {
                    name
                } else {
                    return Err(CompErr::new_cannot_assign_special_ident(name));
                }
            }
            _ => return Err(CompErr::new_expected_ident()),
        };
        self.push(Inst::DeclareVar(name.clone()));
        self.visit_expr(value_expr, Some(name.clone()))?;
        self.push(Inst::AssignVar(name));
        Ok(())
    }

    fn visit_compare_op(
        &mut self,
        expr_a: ast::Expr,
        op: CompareOperator,
        expr_b: ast::Expr,
    ) -> VisitResult {
        self.visit_expr(expr_a, None)?;
        self.visit_expr(expr_b, None)?;
        self.push(Inst::CompareOp(op));
        Ok(())
    }

    fn visit_inplace_op(
        &mut self,
        expr_a: ast::Expr,
        op: InplaceOperator,
        expr_b: ast::Expr,
    ) -> VisitResult {
        if expr_a.is_ident().is_none() {
            return Err(CompErr::new_expected_ident());
        }
        self.visit_expr(expr_a, None)?;
        self.visit_expr(expr_b, None)?;
        self.push(Inst::InplaceOp(op));
        Ok(())
    }

    // Utilities -------------------------------------------------------

    fn len(&self) -> usize {
        self.code.len_chunk()
    }

    fn push(&mut self, inst: Inst) {
        self.code.push_inst(inst);
    }

    // Global constants ------------------------------------------------

    fn push_nil(&mut self) {
        self.push(Inst::LoadNil)
    }

    fn push_true(&mut self) {
        self.push(Inst::LoadTrue)
    }

    fn push_false(&mut self) {
        self.push(Inst::LoadFalse)
    }

    // Code unit constants ---------------------------------------------

    fn add_const(&mut self, val: ObjectRef) {
        let index = self.code.add_const(val);
        self.push(Inst::LoadConst(index));
    }

    // Scopes ----------------------------------------------------------

    /// Add nested scope to current scope then make the new scope the
    /// current scope.
    fn enter_scope(&mut self, kind: ScopeKind) {
        self.scope_tree.add(kind);
        self.scope_depth += 1;
    }

    /// Move up to the parent scope of the current scope.
    fn exit_scope(&mut self) {
        self.scope_tree.move_up();
        self.scope_depth -= 1;
    }

    /// Update jump instructions with their target label addresses.
    fn fix_jumps(&mut self) -> VisitResult {
        let code = &mut self.code;
        let scope_tree = &self.scope_tree;
        let mut not_found: Option<String> = None;
        let mut jump_out_of_func: Option<String> = None;
        scope_tree.walk_up(&mut |scope: &Scope, jump_depth: usize| {
            for (name, jump_addr) in scope.jumps().iter() {
                let result = scope.find_label(scope_tree, name, None);
                if let Some((label_addr, label_depth)) = result {
                    let depth = jump_depth - label_depth;
                    code.replace_inst(*jump_addr, Inst::JumpPushNil(label_addr, depth));
                } else {
                    if scope.kind == ScopeKind::Func {
                        jump_out_of_func = Some(name.clone());
                    } else {
                        not_found = Some(name.clone());
                    }
                    return false;
                }
            }
            true
        });
        if let Some(name) = jump_out_of_func {
            return Err(CompErr::new_cannot_jump_out_of_func(name));
        } else if let Some(name) = not_found {
            return Err(CompErr::new_label_not_found_in_scope(name));
        }
        Ok(())
    }
}
