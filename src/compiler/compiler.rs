//! Compiler.
use std::collections::HashSet;

use crate::ast;
use crate::modules::std::STD;
use crate::types::{new, Module};
use crate::util::Stack;
use crate::vm::{Code, Inst};

use super::result::{CompErr, CompResult, VisitResult};
use super::visitor::Visitor;

// Compiler ------------------------------------------------------------

struct CaptureInfo {
    name: String,
    free_var_addr: usize,
    found_stack_index: usize,
    // Cell vars in enclosing functions. These are discovered while
    // processing the free vars of the current function (assuming it's
    // an inner function).
    cell_var_assignments: Vec<usize>, // address
    cell_var_loads: Vec<usize>,       // address
}

pub struct Compiler {
    // The visitor stack is analogous to the VM call stack.
    visitor_stack: Stack<(Visitor, usize)>, // visitor, scope tree pointer
    // Known global names. This can be used in contexts where globals
    // are known to exist but aren't available to the compiler (e.g., in
    // the REPL).
    global_names: HashSet<String>,
}

impl Default for Compiler {
    fn default() -> Self {
        let mut global_names = HashSet::default();
        global_names.insert("$full_name".to_owned());
        global_names.insert("$name".to_owned());
        global_names.insert("$path".to_owned());
        global_names.insert("$doc".to_owned());
        Self::new(global_names)
    }
}

impl Compiler {
    pub fn new(global_names: HashSet<String>) -> Self {
        Self { visitor_stack: Stack::new(), global_names }
    }

    /// Compile AST module node to module object.
    pub fn compile_module(
        &mut self,
        name: &str,
        file_name: &str,
        ast_module: ast::Module,
    ) -> CompResult {
        let code = self.compile_module_to_code(name, ast_module)?;
        Ok(Module::new(name.to_owned(), file_name.to_owned(), code, None))
    }

    /// Compile AST module node to code object.
    pub fn compile_module_to_code(
        &mut self,
        module_name: &str,
        module: ast::Module,
    ) -> Result<Code, CompErr> {
        let mut visitor = Visitor::for_module(module_name, self.global_names.clone());
        visitor.visit_module(module)?;
        self.global_names = self
            .global_names
            .union(&visitor.scope_tree.global_names())
            .cloned()
            .collect();
        assert!(
            visitor.scope_tree.in_global_scope(),
            "Expected to be in global scope after compiling module"
        );
        // Compile global functions
        let func_nodes = visitor.func_nodes.to_vec();
        self.visitor_stack.push((visitor, 0));
        for (func_name, addr, scope_tree_pointer, node) in func_nodes {
            self.compile_func(
                module_name,
                func_name.as_str(),
                addr,
                scope_tree_pointer,
                node,
            )?;
        }
        let mut visitor = self.visitor_stack.pop().unwrap().0;
        // XXX: This keeps the stack clean and ensures there's always a
        //      jump target at the end of the module.
        visitor.push(Inst::Pop);
        Ok(visitor.code)
    }

    /// Compile AST function node and inject it into the *parent*
    /// visitor at the specified address.
    fn compile_func(
        &mut self,
        module_name: &str,
        func_name: &str,
        // Address in parent visitor's code where function was defined.
        func_addr: usize,
        // Pointer to scope in parent where function was defined. This
        // is needed so that we can start the search for cell vars in
        // the correct scope in the parent visitor.
        parent_scope_pointer: usize,
        node: ast::Func,
    ) -> VisitResult {
        let stack = &mut self.visitor_stack;
        let params = node.params.clone();

        let mut visitor = Visitor::for_func(func_name, self.global_names.clone());
        visitor.visit_func(node)?;

        // Unresolved names are assumed to be globals or builtins.
        let mut presumed_globals = vec![];

        // Captured vars in current function. These are free vars in the
        // current function that are defined in an enclosing function or
        // module.
        //
        // Each entry contains the address of the free var in the
        // current function and the local var index in the enclosing
        // function.
        let mut captured: Vec<CaptureInfo> = vec![];

        for (free_var_addr, name, start, end) in visitor.code.free_vars().iter() {
            let mut found = false;
            let mut found_stack_index = stack.len();
            let mut current_scope_pointer = parent_scope_pointer;

            for (up_visitor, up_scope_pointer) in stack.iter() {
                found_stack_index -= 1;

                // XXX: Don't capture globals (they're handled below).
                if up_visitor.in_global_scope() {
                    break;
                }

                let result = up_visitor
                    .scope_tree
                    .find_var(name.as_str(), Some(current_scope_pointer));

                if let Some(cell_var) = result {
                    let cell_var_addr = cell_var.addr;

                    found = true;

                    let mut info = CaptureInfo {
                        name: name.clone(),
                        free_var_addr: *free_var_addr,
                        found_stack_index,
                        cell_var_assignments: vec![],
                        cell_var_loads: vec![],
                    };

                    for (addr, inst) in up_visitor.code.iter_chunk().enumerate() {
                        if addr >= cell_var_addr {
                            match inst {
                                Inst::ScopeEnd => {
                                    break;
                                }
                                Inst::AssignVar(n) if n == name => {
                                    info.cell_var_assignments.push(addr);
                                }
                                Inst::LoadVar(n, 0) if n == name => {
                                    info.cell_var_loads.push(addr);
                                }
                                _ => (),
                            }
                        }
                    }

                    captured.push(info);

                    break;
                }

                current_scope_pointer = *up_scope_pointer;
            }

            if !found {
                presumed_globals.push((*free_var_addr, name.to_owned(), *start, *end));
            }
        }

        let std = STD.read().unwrap();
        for (addr, name, start, end) in presumed_globals.into_iter() {
            if self.global_names.contains(&name) {
                visitor.replace(addr, Inst::LoadGlobal(name));
            } else if std.has_global(&name) {
                visitor.replace(addr, Inst::LoadBuiltin(name));
            } else {
                return Err(CompErr::name_not_found(name, start, end));
            }
        }

        for info in captured.iter() {
            let name = info.name.as_str();
            let found_stack_index = info.found_stack_index;

            // Update LOAD_VAR instructions in current visitor /
            // function to load free vars from captured cells.
            visitor.replace(info.free_var_addr, Inst::LoadCaptured(name.to_string()));

            // Update ASSIGN_VAR instructions in upward visitor to
            // assign into cell.
            for addr in info.cell_var_assignments.iter() {
                let up_visitor = &mut stack[found_stack_index].0;
                up_visitor.replace(*addr, Inst::AssignCell(name.to_owned()));
            }

            // Update LOAD_VAR instructions in upward visitor to load
            // from cell.
            for addr in info.cell_var_loads.iter() {
                let up_visitor = &mut stack[found_stack_index].0;
                up_visitor.replace(*addr, Inst::LoadCell(name.to_owned()));
            }

            // Update intermediate closures to capture vars so they can
            // be passed down.
            for i in info.found_stack_index..stack.len() {
                let up_visitor = &mut stack[i].0;

                let mut replacements = vec![];
                for (addr, inst) in up_visitor.code.iter_chunk().enumerate() {
                    if let Inst::CaptureSet(names) = inst {
                        if !names.iter().any(|n| n == name) {
                            replacements.push((addr, names.to_vec()));
                        }
                    }
                }

                for (addr, mut names) in replacements {
                    names.push(name.to_owned());
                    up_visitor.replace(addr, Inst::CaptureSet(names));
                }
            }
        }

        // Inner Functions ---------------------------------------------

        let inner_func_nodes = visitor.func_nodes.to_vec();
        self.visitor_stack.push((visitor, parent_scope_pointer));
        for (func_name, addr, scope_tree_pointer, node) in inner_func_nodes {
            self.compile_func(
                module_name,
                func_name.as_str(),
                addr,
                scope_tree_pointer,
                node,
            )?;
        }
        let visitor = self.visitor_stack.pop().unwrap().0;

        // END Inner Functions -----------------------------------------

        let func = new::func(module_name, func_name, params, visitor.code);
        let parent_visitor = &mut self.visitor_stack.peek_mut().unwrap().0;
        let const_index = parent_visitor.code.add_const(func);

        parent_visitor.replace(func_addr, Inst::LoadConst(const_index));

        Ok(())
    }
}
