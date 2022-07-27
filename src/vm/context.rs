use std::slice::Iter;
use std::sync::{Arc, Mutex};

use crate::types::{BuiltinFn, Builtins, ObjectRef, BUILTIN_TYPES};

use super::namespace::Namespace;
use super::objects::Objects;
use super::result::RuntimeErr;

pub struct RuntimeContext {
    pub builtins: Builtins,
    constants: Objects,
    namespace_stack: Vec<Namespace>,
}

impl RuntimeContext {
    pub fn new(
        builtins: Builtins,
        constants: Objects,
        namespace_stack: Vec<Namespace>,
    ) -> Self {
        Self { builtins, constants, namespace_stack }
    }

    pub fn iter_constants(&self) -> Iter<'_, ObjectRef> {
        self.constants.iter()
    }

    fn current_namespace(&mut self) -> &mut Namespace {
        let index = self.depth();
        &mut self.namespace_stack[index]
    }

    fn global_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace_stack[0]
    }

    pub fn enter_scope(&mut self) {
        let mut namespace = Namespace::new();
        namespace.add_obj(self.builtins.nil_obj.clone());
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

    pub fn depth(&self) -> usize {
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

    // Objects ---------------------------------------------------------
    //
    // Objects are allocated dynamically in the current scope and are
    // collected when the current scope exits.

    pub fn add_obj(&mut self, obj: ObjectRef) -> usize {
        let namespace = self.current_namespace();
        namespace.add_obj(obj)
    }

    // Vars ------------------------------------------------------------

    /// Declare a new var in the current namespace. This adds a slot for
    /// the var in the current namespace and sets its initial value to
    /// nil.
    pub fn declare_var(&mut self, name: &str) -> Result<usize, RuntimeErr> {
        let namespace = self.current_namespace();
        namespace.add_var(name)
    }

    /// Assign value to var. This looks up the var by name in the
    /// current namespace and updates its value.
    pub fn assign_var(
        &mut self,
        name: &str,
        obj: ObjectRef,
    ) -> Result<(usize, usize), RuntimeErr> {
        let namespace = self.current_namespace();
        let index = namespace.set_var(name, obj)?;
        Ok((self.depth(), index))
    }

    /// Assign value to var--reach into the namespace at depth and set
    /// the var at the specified index.
    pub fn assign_var_by_depth_and_index(
        &mut self,
        depth: usize,
        index: usize,
        obj: ObjectRef,
    ) -> Result<usize, RuntimeErr> {
        self.namespace_stack[depth].set_obj(index, obj)
    }

    /// Get var from current namespace.
    pub fn get_var_in_current_namespace(
        &mut self,
        name: &str,
    ) -> Result<&ObjectRef, RuntimeErr> {
        let namespace = self.current_namespace();
        namespace.get_var(name)
    }

    /// Reach into the namespace at depth and get the var at the
    /// specified index.
    pub fn get_var_by_depth_and_index(
        &self,
        depth: usize,
        index: usize,
    ) -> Result<&ObjectRef, RuntimeErr> {
        self.namespace_stack[depth].get_obj(index)
    }

    /// Find the a var by name in the current namespace or a parent
    /// namespace, returning the depth where it was found and the index
    /// the containing namespace's object storage.
    pub fn var_index(&mut self, name: &str) -> Result<(usize, usize), RuntimeErr> {
        let mut depth = self.depth();
        loop {
            let namespace = &self.namespace_stack[depth];
            if let Ok(index) = namespace.var_index(name) {
                break Ok((depth, index));
            }
            if depth == 0 {
                let message = format!("Name not found: {name}");
                break Err(RuntimeErr::new_name_err(message));
            }
            depth -= 1;
        }
    }

    fn add_builtin_types(&mut self) -> Result<(), RuntimeErr> {
        for (name, class) in BUILTIN_TYPES.iter() {
            self.declare_var(name)?;
            self.assign_var(name, class.clone())?;
        }
        Ok(())
    }

    fn add_builtin_func(
        &mut self,
        name: &str,
        func: BuiltinFn,
        arity: Option<u8>,
    ) -> Result<(), RuntimeErr> {
        let func = self.builtins.new_builtin_func(name, func, arity);
        self.declare_var(name)?;
        self.assign_var(name, func)?;
        Ok(())
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        let builtins = Builtins::new();

        // Singletons
        let nil_obj = builtins.nil_obj.clone();
        let true_obj = builtins.true_obj.clone();
        let false_obj = builtins.false_obj.clone();

        let constants = Objects::default();
        let namespace_stack = vec![];
        let mut ctx = RuntimeContext::new(builtins, constants, namespace_stack);

        // Add builtin types to builtins namespace.
        let mut builtins_ns = crate::types::Namespace::new("builtins");
        builtins_ns.add_obj(nil_obj.clone());
        for (name, class) in BUILTIN_TYPES.iter() {
            if builtins_ns.add_var(*name).is_none() {
                panic!("Could not add {name} to {builtins_ns}");
            }
            if builtins_ns.set_var(name, class.clone()).is_none() {
                panic!("Could not set {name} in {builtins_ns}");
            }
        }

        // Add singleton constants.
        ctx.add_const(nil_obj); // 0
        ctx.add_const(true_obj); // 1
        ctx.add_const(false_obj); // 2

        // Enter the global scope.
        // NOTE: This needs to be done before adding global vars.
        ctx.enter_scope();

        // Add builtins var to global scope.
        // NOTE: This needs to be done after entering the global scope.
        let builtins_ns_var = Arc::new(Mutex::new(builtins_ns));
        if let Err(err) = ctx.declare_var("builtins") {
            panic!("Could not declare global builtins var: {err}");
        }
        if let Err(err) = ctx.assign_var("builtins", builtins_ns_var) {
            panic!("Could not declare global builtins var: {err}");
        }

        // Add shorthand aliases for builtin types to the global
        // namespace.
        for (name, class) in BUILTIN_TYPES.iter() {
            if let Err(err) = ctx.declare_var(name) {
                panic!("Could not declare global var {name}: {err}");
            }
            if let Err(err) = ctx.assign_var(name, class.clone()) {
                panic!("Could not assign global var {name}: {err}");
            }
        }

        // Add builtin functions to global scope
        {
            use crate::builtin_funcs::*;
            let results = [
                // File
                ctx.add_builtin_func("read_file", read_file, Some(1)),
                ctx.add_builtin_func("read_file_lines", read_file_lines, Some(1)),
                // Print
                ctx.add_builtin_func("print", print, None),
                // Type
                ctx.add_builtin_func("type_of", type_of, None),
                ctx.add_builtin_func("obj_id", obj_id, None),
            ];
            for result in results {
                match result {
                    Ok(_) => (),
                    Err(err) => panic!("Could not create builtin function: {err}"),
                }
            }
        }

        ctx
    }
}
