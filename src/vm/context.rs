use std::collections::hash_map;
use std::slice;

use crate::types::{create, Namespace, ObjectRef, ObjectTrait, BUILTINS};

use super::constants::Constants;
use super::result::{RuntimeErr, RuntimeResult};

pub struct RuntimeContext {
    constants: Constants,
    namespace_stack: Vec<Namespace>,
    // Number of namespaces on stack
    size: usize,
    // Index of namespace associated with current scope
    current_depth: usize,
    pub argv: Vec<String>,
}

impl Default for RuntimeContext {
    fn default() -> Self {
        RuntimeContext::new(vec![])
    }
}

impl RuntimeContext {
    pub fn new(argv: Vec<&str>) -> Self {
        let mut ctx = Self {
            constants: Constants::default(),
            namespace_stack: vec![],
            size: 0,
            current_depth: 0,
            argv: argv.into_iter().map(|a| a.to_owned()).collect(),
        };
        ctx.init();
        ctx
    }

    fn init(&mut self) {
        // Add singleton constants.
        self.add_const(create::new_nil()); // 0
        self.add_const(create::new_true()); // 1
        self.add_const(create::new_false()); // 2

        // Enter global scope.
        self.enter_scope();

        // Builtins ----------------------------------------------------

        // Add builtins module to global scope.
        let builtins = BUILTINS.clone();
        if let Err(err) = self.declare_and_assign_var("builtins", builtins) {
            panic!("Could not define builtins module: {err}");
        }

        // Add shorthand aliases for builtin types and objects to global
        // scope.
        let builtins = BUILTINS.clone();
        let reader = builtins.read().unwrap();
        let ns = reader.namespace();
        for (name, obj) in ns.iter().filter(|(n, _)| !n.starts_with('$')) {
            if let Err(err) = self.declare_and_assign_var(name, (*obj).clone()) {
                panic!("Could not add alias for builtin object `{name}` to global scope: {err}");
            }
        }
    }

    pub fn iter_constants(&self) -> slice::Iter<'_, ObjectRef> {
        self.constants.iter()
    }

    pub fn iter_vars(&self) -> hash_map::Iter<'_, String, ObjectRef> {
        self.namespace_stack[self.current_depth].iter()
    }

    #[inline]
    fn current_namespace(&mut self) -> &mut Namespace {
        &mut self.namespace_stack[self.current_depth]
    }

    pub fn enter_scope(&mut self) {
        self.current_depth = self.size;
        self.namespace_stack.push(Namespace::new());
        self.size += 1;
    }

    pub fn exit_scope(&mut self) {
        if self.current_depth == 0 {
            panic!("Can't remove global namespace");
        }
        let mut ns = self.namespace_stack.pop().expect("Expected namespace");
        ns.clear();
        self.size -= 1;
        self.current_depth -= 1;
    }

    pub fn exit_scopes(&mut self, count: usize) {
        for _ in 0..count {
            self.exit_scope();
        }
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
        let initial = create::new_nil();
        let namespace = self.current_namespace();
        namespace.add_obj(name, initial);
    }

    /// Assign value to var. This looks up the var by name in the
    /// current namespace and updates its value.
    pub fn assign_var(
        &mut self,
        name: &str,
        obj: ObjectRef,
    ) -> Result<usize, RuntimeErr> {
        let namespace = self.current_namespace();
        if namespace.set_obj(name, obj) {
            Ok(self.current_depth)
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
        if self.namespace_stack[depth].set_obj(name, obj) {
            Ok(())
        } else {
            let message = format!("Name not defined at depth {depth}: {name}");
            Err(RuntimeErr::new_name_err(message))
        }
    }

    /// Get the depth of the namespace where the specified var is
    /// defined.
    pub fn get_var_depth(&mut self, name: &str) -> Result<usize, RuntimeErr> {
        let mut var_depth = self.current_depth;
        loop {
            let namespace = &self.namespace_stack[var_depth];
            if namespace.get_obj(name).is_some() {
                break Ok(var_depth);
            }
            if var_depth == 0 {
                let message = format!("Name not found: {name}");
                break Err(RuntimeErr::new_name_err(message));
            }
            var_depth -= 1;
        }
    }

    /// Get var from current namespace.
    pub fn get_var_in_current_namespace(
        &mut self,
        name: &str,
    ) -> Result<ObjectRef, RuntimeErr> {
        let namespace = self.current_namespace();
        if let Some(obj) = namespace.get_obj(name) {
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
    ) -> Result<ObjectRef, RuntimeErr> {
        if let Some(obj) = self.namespace_stack[depth].get_obj(name) {
            Ok(obj)
        } else {
            let message = format!("Name not defined at depth {depth}: {name}");
            Err(RuntimeErr::new_name_err(message))
        }
    }
}
