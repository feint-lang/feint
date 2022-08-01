use std::slice::Iter;
use std::sync::Arc;

use crate::builtin_funcs::get_builtin_func_specs;
use crate::types::{Builtins, Namespace, ObjectRef, BUILTIN_TYPES};

use super::objects::Objects;
use super::result::{RuntimeErr, RuntimeResult};

pub struct RuntimeContext {
    pub builtins: Builtins,
    constants: Objects,
    namespace_stack: Vec<Namespace>,
    pub argv: Vec<String>,
}

impl RuntimeContext {
    pub fn new(
        builtins: Builtins,
        constants: Objects,
        namespace_stack: Vec<Namespace>,
        argv: Vec<String>,
    ) -> Self {
        Self { builtins, constants, namespace_stack, argv }
    }

    pub fn with_argv(argv: Vec<&str>) -> Self {
        let mut ctx = Self::default();
        let mut owned_argv: Vec<String> = vec![];
        for arg in argv.iter() {
            owned_argv.push(arg.to_string());
        }
        ctx.argv = owned_argv;
        ctx
    }

    pub fn iter_constants(&self) -> Iter<'_, ObjectRef> {
        self.constants.iter()
    }

    fn current_namespace(&mut self) -> &mut Namespace {
        let index = self.depth();
        &mut self.namespace_stack[index]
    }

    pub fn enter_scope(&mut self) {
        let namespace = Namespace::new();
        self.namespace_stack.push(namespace);
    }

    pub fn exit_scope(&mut self) {
        if self.depth() == 0 {
            panic!("Can't remove global namespace");
        }
        if let Some(mut namespace) = self.namespace_stack.pop() {
            namespace.clear();
        }
    }

    pub fn exit_scopes(&mut self, count: usize) {
        for _ in 0..count {
            self.exit_scope();
        }
    }

    fn depth(&self) -> usize {
        self.namespace_stack.len() - 1
    }

    // Constants -------------------------------------------------------
    //
    // Constants are allocated during compilation, are immutable, and
    // are never collected.

    pub fn add_const(&mut self, obj: ObjectRef) -> usize {
        self.constants.add(obj)
    }

    pub fn get_const(&self, index: usize) -> Result<&ObjectRef, RuntimeErr> {
        self.constants.get(index)
    }

    // Vars ------------------------------------------------------------

    /// Declare a new var in the current namespace. This adds a slot for
    /// the var in the current namespace and sets its initial value to
    /// nil.
    pub fn declare_var(&mut self, name: &str) {
        let initial = self.builtins.nil_obj.clone();
        let namespace = self.current_namespace();
        namespace.add_entry(name, initial);
    }

    /// Assign value to var. This looks up the var by name in the
    /// current namespace and updates its value.
    pub fn assign_var(
        &mut self,
        name: &str,
        obj: ObjectRef,
    ) -> Result<usize, RuntimeErr> {
        let namespace = self.current_namespace();
        if namespace.set_entry(name, obj) {
            Ok(self.depth())
        } else {
            let message = format!("Name not defined in current namespace: {name}");
            Err(RuntimeErr::new_name_err(message))
        }
    }

    /// Conveniently declare and assign a var in one step.
    pub fn declare_and_assign_var(
        &mut self,
        name: &str,
        obj: ObjectRef,
    ) -> Result<usize, RuntimeErr> {
        self.declare_var(name);
        self.assign_var(name, obj)
    }

    /// Assign value to var--reach into the namespace at depth and set
    /// the var at the specified index.
    pub fn assign_var_at_depth(
        &mut self,
        depth: usize,
        name: &str,
        obj: ObjectRef,
    ) -> RuntimeResult {
        if self.namespace_stack[depth].set_entry(name, obj) {
            Ok(())
        } else {
            let message = format!("Name not defined at depth {depth}: {name}");
            Err(RuntimeErr::new_name_err(message))
        }
    }

    /// Get the depth of the namespace where the specified var is
    /// defined.
    pub fn get_var_depth(&mut self, name: &str) -> Result<usize, RuntimeErr> {
        let mut depth = self.depth();
        loop {
            let namespace = &self.namespace_stack[depth];
            if let Some(_) = namespace.get_entry(name) {
                break Ok(depth);
            }
            if depth == 0 {
                let message = format!("Name not found: {name}");
                break Err(RuntimeErr::new_name_err(message));
            }
            depth -= 1;
        }
    }

    /// Get var from current namespace.
    pub fn get_var_in_current_namespace(
        &mut self,
        name: &str,
    ) -> Result<&ObjectRef, RuntimeErr> {
        let namespace = self.current_namespace();
        if let Some(obj) = namespace.get_entry(name) {
            Ok(obj)
        } else {
            let message = format!("Name not defined in current namespace: {name}");
            Err(RuntimeErr::new_name_err(message))
        }
    }

    /// Reach into the namespace at depth and get the specified var.
    pub fn get_var_at_depth(
        &self,
        depth: usize,
        name: &str,
    ) -> Result<&ObjectRef, RuntimeErr> {
        if let Some(obj) = self.namespace_stack[depth].get_entry(name) {
            Ok(obj)
        } else {
            let message = format!("Name not defined at depth {depth}: {name}");
            Err(RuntimeErr::new_name_err(message))
        }
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        let mut ctx =
            RuntimeContext::new(Builtins::new(), Objects::default(), vec![], vec![]);

        // Add singleton constants.
        ctx.add_const(ctx.builtins.nil_obj.clone()); // 0
        ctx.add_const(ctx.builtins.true_obj.clone()); // 1
        ctx.add_const(ctx.builtins.false_obj.clone()); // 2

        // Enter global scope.
        ctx.enter_scope();

        // Add builtin types to builtins namespace and add aliases to
        // global scope.
        let mut builtins_ns = Namespace::new();
        for (name, class) in BUILTIN_TYPES.iter() {
            builtins_ns.add_entry(*name, class.clone());
            if let Err(err) = ctx.declare_and_assign_var(name, class.clone()) {
                panic!("Could not define builtin type {name}: {err}");
            }
        }

        // Add builtin functions to builtins namespace and add aliases
        // to global scope.
        for spec in get_builtin_func_specs() {
            let (name, params, func) = spec;
            let func = ctx.builtins.new_builtin_func(name, params, func, None);
            builtins_ns.add_entry(name, func.clone());
            if let Err(err) = ctx.declare_and_assign_var(name, func.clone()) {
                panic!("Could not define builtin func: {err}");
            }
        }

        // Add builtins namespace to global scope.
        let builtins_ns_var = Arc::new(builtins_ns);
        if let Err(err) = ctx.declare_and_assign_var("builtins", builtins_ns_var) {
            panic!("Could not define global builtins var: {err}");
        }

        ctx
    }
}
