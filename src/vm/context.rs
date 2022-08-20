use std::collections::hash_map;
use std::slice;

use crate::modules;
use crate::types::{create, Namespace, ObjectRef, ObjectTrait};

use super::constants::Constants;
use super::result::{RuntimeErr, RuntimeResult};

pub struct RuntimeContext {
    global_constants: Constants,
    namespace_stack: Vec<Namespace>,
    // Number of namespaces on stack
    size: usize,
    // Index of namespace associated with current scope
    current_depth: usize,
}

impl RuntimeContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            global_constants: Constants::new(),
            namespace_stack: vec![],
            size: 0,
            current_depth: 0,
        };
        ctx.init();
        ctx
    }

    fn init(&mut self) {
        // Add singleton constants.
        self.add_global_const(create::new_nil()); // 0
        self.add_global_const(create::new_true()); // 1
        self.add_global_const(create::new_false()); // 2

        for int in create::SHARED_INTS.iter() {
            self.add_global_const(int.clone());
        }

        // Enter global scope.
        self.enter_scope();

        // Builtins ----------------------------------------------------

        // Add builtins module to global scope.
        let builtins = modules::BUILTINS.clone();
        if let Err(err) = self.declare_and_assign_var("builtins", builtins) {
            panic!("Could not define builtins module: {err}");
        }

        // Add shorthand aliases for builtin types and objects to global
        // scope.
        let builtins = modules::BUILTINS.clone();
        let reader = builtins.read().unwrap();
        let ns = reader.namespace();
        for (name, obj) in ns.iter().filter(|(n, _)| !n.starts_with('$')) {
            if let Err(err) = self.declare_and_assign_var(name, (*obj).clone()) {
                panic!("Could not add alias for builtin object `{name}` to global scope: {err}");
            }
        }
    }

    #[inline]
    fn current_ns(&self) -> &Namespace {
        &self.namespace_stack[self.current_depth]
    }

    fn current_ns_mut(&mut self) -> &mut Namespace {
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

    pub fn exit_all_scopes(&mut self) {
        while self.current_depth != 0 {
            self.exit_scope();
        }
    }

    // Global Constants ------------------------------------------------
    //
    // Global constants are allocated during compilation, are immutable,
    // and are never collected. These are shared constants such as the
    // singleton nil, true, and false objects.

    pub fn add_global_const(&mut self, obj: ObjectRef) -> usize {
        self.global_constants.add(obj)
    }

    pub fn get_global_const(&self, index: usize) -> Result<&ObjectRef, RuntimeErr> {
        self.global_constants.get(index)
    }

    pub fn iter_constants(&self) -> slice::Iter<'_, ObjectRef> {
        self.global_constants.iter()
    }

    // Vars ------------------------------------------------------------

    /// Declare a new var in the current namespace. This adds a slot for
    /// the var in the current namespace and sets its initial value to
    /// nil.
    pub fn declare_var(&mut self, name: &str) {
        let initial = create::new_nil();
        let namespace = self.current_ns_mut();
        namespace.add_obj(name, initial);
    }

    /// Assign value to var in *current* namespace. This looks up the
    /// var by name in the current namespace, updates its value, and
    /// returns the depth of the namespace where the var lives. If the
    /// var doesn't exist in the current namespace, an error is returned
    /// instead (indicating an internal error).
    pub fn assign_var(
        &mut self,
        name: &str,
        obj: ObjectRef,
    ) -> Result<usize, RuntimeErr> {
        let namespace = self.current_ns_mut();
        if namespace.set_obj(name, obj) {
            Ok(self.current_depth)
        } else {
            let message = format!("Name not defined in current namespace: {name}");
            Err(RuntimeErr::name_err(message))
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
            Err(RuntimeErr::name_err(message))
        }
    }

    /// Get depth of namespace where var is defined.
    pub fn get_var_depth(
        &self,
        name: &str,
        starting_depth: Option<usize>,
    ) -> Result<usize, RuntimeErr> {
        let ns_stack = &self.namespace_stack;
        let mut var_depth =
            if let Some(depth) = starting_depth { depth } else { self.current_depth };
        loop {
            if ns_stack[var_depth].get_obj(name).is_some() {
                break Ok(var_depth);
            }
            if var_depth == 0 {
                let message = format!("Name not found: {name}");
                break Err(RuntimeErr::name_err(message));
            }
            var_depth -= 1;
        }
    }

    /// Get depth of namespace where outer var is defined (skips current
    /// namespace, starts search from parent namespace).
    pub fn get_outer_var_depth(&self, name: &str) -> Result<usize, RuntimeErr> {
        if self.current_depth == 0 {
            let message = format!("Name not found: {name}");
            return Err(RuntimeErr::name_err(message));
        }
        self.get_var_depth(name, Some(self.current_depth - 1))
    }

    /// Get var from current namespace.
    pub fn get_var_in_current_namespace(
        &self,
        name: &str,
    ) -> Result<ObjectRef, RuntimeErr> {
        let namespace = self.current_ns();
        if let Some(obj) = namespace.get_obj(name) {
            Ok(obj)
        } else {
            let message = format!("Name not defined in current namespace: {name}");
            Err(RuntimeErr::name_err(message))
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
            Err(RuntimeErr::name_err(message))
        }
    }

    /// Get var in current namespace or any ancestor namespace.
    pub fn get_var(&self, name: &str) -> Result<ObjectRef, RuntimeErr> {
        let depth = self.get_var_depth(name, None)?;
        self.get_var_at_depth(depth, name)
    }

    /// Get var in parent namespace or any ancestor of the parent
    /// namespace.
    pub fn get_outer_var(&self, name: &str) -> Result<ObjectRef, RuntimeErr> {
        let depth = self.get_outer_var_depth(name)?;
        self.get_var_at_depth(depth, name)
    }

    pub fn iter_vars(&self) -> hash_map::Iter<'_, String, ObjectRef> {
        self.namespace_stack[self.current_depth].iter()
    }
}
