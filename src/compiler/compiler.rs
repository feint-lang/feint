use crate::ast;
use crate::types::ObjectRef;
use crate::util::{BinaryOperator, UnaryOperator};
use crate::vm::{Chunk, Inst, RuntimeContext, VM};

use super::result::{CompilationErr, CompilationErrKind, CompilationResult};
use super::scope::{Scope, ScopeKind, ScopeTree};

// Compiler ------------------------------------------------------------

/// Compile AST to VM instructions.
pub fn compile(vm: &mut VM, program: ast::Program) -> CompilationResult {
    let mut visitor = Visitor::new(&mut vm.ctx);
    visitor.visit_program(program)?;
    Ok(visitor.instructions)
}

// Visitor -------------------------------------------------------------

type VisitResult = Result<(), CompilationErr>;

struct Visitor<'a> {
    ctx: &'a mut RuntimeContext,
    instructions: Chunk,
    scope_tree: ScopeTree,
}

impl<'a> Visitor<'a> {
    fn new(ctx: &'a mut RuntimeContext) -> Self {
        Self { ctx, instructions: Chunk::new(), scope_tree: ScopeTree::new() }
    }

    // Utilities -------------------------------------------------------

    fn err(&self, message: String) -> VisitResult {
        Err(CompilationErr::new(CompilationErrKind::VisitErr(message)))
    }

    fn push(&mut self, inst: Inst) {
        self.instructions.push(inst);
    }

    fn push_const(&mut self, index: usize) {
        self.push(Inst::LoadConst(index));
    }

    fn add_const(&mut self, val: ObjectRef) {
        let index = self.ctx.constants.add(val);
        self.push_const(index);
    }

    /// Add nested scope to current scope then make the new scope the
    /// current scope.
    fn enter_scope(&mut self, kind: ScopeKind) {
        self.scope_tree.add(kind);
    }

    /// Move up to the parent scope of the current scope.
    fn exit_scope(&mut self) {
        self.scope_tree.move_up();
    }

    /// Update jump instructions with their target label addresses.
    fn fix_jumps(&mut self) -> VisitResult {
        let instructions = &mut self.instructions;
        let scope_tree = &self.scope_tree;
        let mut not_found: Option<String> = None;
        scope_tree.walk_up(&mut |scope: &Scope, jump_depth: usize| {
            for (name, jump_addr) in scope.jumps().iter() {
                let result = scope.find_label(scope_tree, name, None);
                if let Some((label_addr, label_depth)) = result {
                    let depth = jump_depth - label_depth;
                    instructions[*jump_addr - 1] = match depth {
                        0 => Inst::NoOp,
                        _ => Inst::ScopeEnd(depth),
                    };
                    instructions[*jump_addr] = Inst::Jump(label_addr);
                } else {
                    not_found = Some(name.clone());
                    return false;
                }
            }
            true
        });
        if let Some(name) = not_found {
            return self.err(format!(
                "Label not found for jump {} (jump target must be *after* jump)",
                name
            ));
        }
        Ok(())
    }

    // Visitors --------------------------------------------------------

    fn visit_program(&mut self, node: ast::Program) -> VisitResult {
        for statement in node.statements {
            self.visit_statement(statement)?;
        }
        assert_eq!(self.scope_tree.pointer(), 0);
        self.fix_jumps()?;
        self.push(Inst::Halt(0));
        Ok(())
    }

    fn visit_block(&mut self, node: ast::Block, scope_kind: ScopeKind) -> VisitResult {
        self.push(Inst::ScopeStart);
        self.enter_scope(scope_kind);
        for statement in node.statements {
            self.visit_statement(statement)?;
        }
        self.push(Inst::ScopeEnd(1));
        self.exit_scope();
        Ok(())
    }

    fn visit_conditional(
        &mut self,
        if_expr: ast::Expr,
        if_block: ast::Block,
        else_block: Option<ast::Block>,
        scope_kind: ScopeKind,
    ) -> VisitResult {
        // Addresses of if and else if Jump out instructions (added to
        // the end of each block). The target address for these isn't
        // known until all the if, else if, and else blocks are
        // compiled.
        let mut jump_out_addrs: Vec<usize> = vec![];

        // Evaluate if expression and leave result on top of stack
        self.visit_expr(if_expr)?;

        // Placeholder to jump to if or else depending on if expr
        let jump_if_else_index = self.instructions.len();
        self.push(Inst::InternalErr("JumpIfElse not set".to_owned()));

        // If block
        let if_addr = jump_if_else_index + 1;
        self.visit_block(if_block, scope_kind.clone())?;

        // Placeholder to jump out of if
        jump_out_addrs.push(self.instructions.len());
        self.push(Inst::InternalErr("Jump for if not set".to_owned()));

        // Else block (if present)
        let else_addr = self.instructions.len();
        if let Some(else_block) = else_block {
            self.visit_block(else_block, scope_kind.clone())?;
        }

        // Address of the next instruction after the conditional suite
        let after_addr = self.instructions.len();

        // Insert jump instructions now that addresses are known
        self.instructions[jump_if_else_index] = Inst::JumpIfElse(if_addr, else_addr);

        for addr in jump_out_addrs {
            self.instructions[addr] = Inst::Jump(after_addr);
        }

        Ok(())
    }

    fn visit_func(&mut self, node: ast::Func) -> VisitResult {
        eprintln!("IMPLEMENT visit_func()!!!");
        eprintln!("{}({})", node.name, node.params.join(", "));
        Ok(())
    }

    fn visit_call(&mut self, node: ast::Call) -> VisitResult {
        eprintln!("IMPLEMENT visit_call()!!!");
        eprintln!("{}()", node.name);
        Ok(())
    }

    fn visit_statement(&mut self, node: ast::Statement) -> VisitResult {
        match node.kind {
            ast::StatementKind::Print(expr) => {
                self.visit_expr(expr)?;
                self.push(Inst::Print);
                self.push(Inst::Push(0));
            }
            ast::StatementKind::Jump(name) => {
                // Insert placeholder jump instruction to be filled in
                // with corresponding label address once labels have
                // been processed. We also take care to exit nested
                // blocks/scopes before jumping out.
                self.push(Inst::ScopeEnd(1));
                self.push(Inst::Jump(0));
                let addr = self.instructions.len() - 1;
                self.scope_tree.add_jump(name.as_str(), addr);
            }
            ast::StatementKind::Label(name) => {
                self.push(Inst::NoOp);
                let addr = self.instructions.len() - 1;
                if self.scope_tree.add_label(name.as_str(), addr).is_some() {
                    self.err(format!("Duplicate label in scope: {}", name))?;
                }
            }
            ast::StatementKind::Expr(expr) => self.visit_expr(expr)?,
        }
        Ok(())
    }

    fn visit_expr(&mut self, node: ast::Expr) -> VisitResult {
        type Kind = ast::ExprKind;
        match node.kind {
            Kind::Block(block) => self.visit_block(block, ScopeKind::Block)?,
            Kind::Conditional(if_expr, if_block, else_block) => self
                .visit_conditional(*if_expr, if_block, else_block, ScopeKind::Block)?,
            Kind::Func(func) => self.visit_func(func)?,
            Kind::UnaryOp(op, b) => self.visit_unary_op(op, *b)?,
            Kind::BinaryOp(a, op, b) => self.visit_binary_op(*a, op, *b)?,
            Kind::Ident(ident) => self.visit_ident(ident)?,
            Kind::Literal(literal) => self.visit_literal(literal)?,
            Kind::Tuple(items) => self.visit_tuple(items)?,
            _ => self.err(format!("Unhandled expression: {:?}", node))?,
        }
        Ok(())
    }

    fn visit_unary_op(&mut self, op: UnaryOperator, expr: ast::Expr) -> VisitResult {
        self.visit_expr(expr)?;
        self.push(Inst::UnaryOp(op));
        Ok(())
    }

    fn visit_binary_op(
        &mut self,
        expr_a: ast::Expr,
        op: BinaryOperator,
        expr_b: ast::Expr,
    ) -> VisitResult {
        if let BinaryOperator::Assign = op {
            self.visit_assignment(expr_a, expr_b)
        } else {
            self.visit_expr(expr_a)?;
            self.visit_expr(expr_b)?;
            self.push(Inst::BinaryOp(op));
            Ok(())
        }
    }

    fn visit_assignment(
        &mut self,
        name_expr: ast::Expr,
        value_expr: ast::Expr,
    ) -> VisitResult {
        match name_expr.kind {
            ast::ExprKind::Ident(ident) => match ident.kind {
                ast::IdentKind::Ident(name) => {
                    // NOTE: Currently, declaration and assignment are
                    //       the same thing, so declaration doesn't
                    //       do anything particularly useful ATM.
                    self.push(Inst::DeclareVar(name.clone()));
                    self.visit_expr(value_expr)?;
                    self.push(Inst::AssignVar(name));
                }
                _ => return self.err("Expected identifier".to_owned()),
            },
            _ => return self.err("Expected identifier".to_owned()),
        }
        Ok(())
    }

    // Visit identifier as expression (i.e., not as part of an
    // assignment).
    fn visit_ident(&mut self, node: ast::Ident) -> VisitResult {
        type Kind = ast::IdentKind;
        match node.kind {
            Kind::Ident(name) => self.push(Inst::LoadVar(name)),
            Kind::TypeIdent(name) => self.push(Inst::LoadVar(name)),
        }
        Ok(())
    }

    fn visit_literal(&mut self, node: ast::Literal) -> VisitResult {
        type Kind = ast::LiteralKind;
        let builtins = &self.ctx.builtins;
        match node.kind {
            Kind::Nil => self.push_const(0),
            Kind::Bool(true) => self.push_const(1),
            Kind::Bool(false) => self.push_const(2),
            Kind::Float(value) => self.add_const(builtins.new_float(value)),
            Kind::Int(value) => self.add_const(builtins.new_int(value)),
            Kind::String(value) => self.add_const(builtins.new_string(value)),
            Kind::FormatString(value) => self.add_const(builtins.new_string(value)),
        }
        Ok(())
    }

    /// XXX: The way this works currently, tuples can only contain
    ///      literal values because there's no way to store a name
    ///      reference as an object. Python uses a different approach--
    ///      a BUILD_TUPLE instruction that would probably fix this.
    fn visit_tuple(&mut self, exprs: Vec<ast::Expr>) -> VisitResult {
        let items = self.convert_tuple_items(exprs);
        self.add_const(self.ctx.builtins.new_tuple(items));
        Ok(())
    }

    fn convert_tuple_items(&self, exprs: Vec<ast::Expr>) -> Vec<ObjectRef> {
        type Kind = ast::LiteralKind;
        let builtins = &self.ctx.builtins;
        let mut items: Vec<ObjectRef> = vec![];
        for expr in exprs {
            let obj: ObjectRef = match expr.kind {
                ast::ExprKind::Literal(literal) => match literal.kind {
                    Kind::Nil => builtins.nil_obj.clone(),
                    Kind::Bool(true) => builtins.true_obj.clone(),
                    Kind::Bool(false) => builtins.false_obj.clone(),
                    Kind::Float(value) => builtins.new_float(value),
                    Kind::Int(value) => builtins.new_int(value),
                    Kind::String(value) => builtins.new_string(value),
                    Kind::FormatString(value) => builtins.new_string(value),
                },
                ast::ExprKind::Tuple(exprs) => {
                    let items = self.convert_tuple_items(exprs);
                    builtins.new_tuple(items)
                }
                ast::ExprKind::Ident(ident) => {
                    unimplemented!("Unhandled identifier in tuple: {:?}", ident)
                }
                ast::ExprKind::Func(func) => {
                    unimplemented!("Unhandled function in tuple: {:?}", func)
                }
                _ => unimplemented!("Unhandled tuple expression: {:?}", expr),
            };
            items.push(obj);
        }
        items
    }
}
