use std::slice::Iter;

use crate::types::{create, Namespace, ObjectRef, ObjectTrait, BUILTINS};

use super::objects::Objects;
use super::result::{RuntimeErr, RuntimeResult};

pub struct RuntimeContext {
    constants: Objects,
    namespace_stack: Vec<Namespace>,
    pub argv: Vec<String>,
}

impl RuntimeContext {
    pub fn new(
        constants: Objects,
        namespace_stack: Vec<Namespace>,
        argv: Vec<String>,
    ) -> Self {
        Self { constants, namespace_stack, argv }
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
        let mut depth = self.depth();
        loop {
            let namespace = &self.namespace_stack[depth];
            if namespace.get_obj(name).is_some() {
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

impl Default for RuntimeContext {
    fn default() -> Self {
        let mut ctx = RuntimeContext::new(Objects::default(), vec![], vec![]);

        // Add singleton constants.
        ctx.add_const(create::new_nil()); // 0
        ctx.add_const(create::new_true()); // 1
        ctx.add_const(create::new_false()); // 2

        // Enter global scope.
        ctx.enter_scope();

        // Builtins ----------------------------------------------------

        let builtins = BUILTINS.clone();

        // Add shorthand aliases for builtin types and objects to global
        // scope.
        for (name, obj) in builtins.namespace().borrow().iter() {
            if !name.starts_with('$') {
                if let Err(err) = ctx.declare_and_assign_var(name, obj.clone()) {
                    panic!(
                        "Could not add alias for builtin object `{name}` to global scope: {err}"
                    );
                }
            }
        }

        // Add builtins module to global scope.
        if let Err(err) = ctx.declare_and_assign_var("builtins", builtins) {
            panic!("Could not define builtins module: {err}");
        }

        ctx
    }
}
