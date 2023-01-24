use crate::modules::std::BUILTINS;
use crate::types::{new, Namespace, ObjectRef, ObjectTrait};
use crate::vm::RuntimeObjResult;

use super::result::{RuntimeErr, RuntimeResult};

pub struct RuntimeContext {
    ns_stack: Vec<Namespace>,
}

impl Default for RuntimeContext {
    fn default() -> Self {
        Self::new(vec![Namespace::default()])
    }
}

impl RuntimeContext {
    pub fn new(ns_stack: Vec<Namespace>) -> Self {
        Self { ns_stack }
    }

    #[inline]
    fn current_ns(&self) -> &Namespace {
        self.ns_stack.last().unwrap()
    }

    #[inline]
    fn current_depth(&self) -> usize {
        self.ns_stack.len() - 1
    }

    fn current_ns_mut(&mut self) -> &mut Namespace {
        self.ns_stack.last_mut().unwrap()
    }

    pub fn globals(&self) -> &Namespace {
        &self.ns_stack[0]
    }

    pub fn enter_scope(&mut self) {
        self.ns_stack.push(Namespace::default());
    }

    pub fn exit_scope(&mut self) {
        if self.current_depth() == 0 {
            panic!("Can't remove global namespace");
        }
        let mut ns = self.ns_stack.pop().expect("Expected namespace");
        ns.clear();
    }

    pub fn exit_all_scopes(&mut self) {
        while self.ns_stack.len() > 1 {
            self.exit_scope();
        }
    }

    // Vars ------------------------------------------------------------

    /// Declare a new var in the current namespace. This adds a slot for
    /// the var in the current namespace and sets its initial value to
    /// nil.
    pub fn declare_var(&mut self, name: &str) {
        let initial = new::nil();
        let ns = self.current_ns_mut();
        ns.add_obj(name, initial);
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
        let ns = self.current_ns_mut();
        if ns.set_obj(name, obj) {
            Ok(self.ns_stack.len() - 1)
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
        if self.ns_stack[depth].set_obj(name, obj) {
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
        offset: usize,
    ) -> Result<usize, RuntimeErr> {
        let ns_stack = &self.ns_stack;
        let mut var_depth = self.current_depth() - offset;
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
    pub fn get_outer_var_depth(
        &self,
        name: &str,
        depth_offset: usize,
    ) -> Result<usize, RuntimeErr> {
        let ns_stack = &self.ns_stack;
        let current_depth = self.current_depth();
        if depth_offset >= current_depth {
            let message = format!("Name not found: {name}");
            return Err(RuntimeErr::name_err(message));
        }
        let mut var_depth = current_depth - depth_offset;
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

    /// Get var from current namespace.
    pub fn get_var_in_current_ns(&self, name: &str) -> RuntimeObjResult {
        let ns = self.current_ns();
        if let Some(obj) = ns.get_obj(name) {
            Ok(obj)
        } else {
            let message = format!("Name not defined in current namespace: {name}");
            Err(RuntimeErr::name_err(message))
        }
    }

    /// Reach into the namespace at depth and get the specified var.
    pub fn get_var_at_depth(&self, depth: usize, name: &str) -> RuntimeObjResult {
        if let Some(obj) = self.ns_stack[depth].get_obj(name) {
            Ok(obj)
        } else {
            let message = format!("Name not defined at depth {depth}: {name}");
            Err(RuntimeErr::name_err(message))
        }
    }

    /// Get var in current namespace or any ancestor namespace.
    pub fn get_var(&self, name: &str, offset: usize) -> RuntimeObjResult {
        let depth = self.get_var_depth(name, offset)?;
        self.get_var_at_depth(depth, name)
    }

    /// Get builtin object. This is used as a fallback when a name isn't
    /// found in the current scope.
    pub fn get_builtin(&self, name: &str) -> ObjectRef {
        let builtins = BUILTINS.read().unwrap();
        let builtins = builtins.down_to_mod().unwrap();
        builtins.get_attr(name, BUILTINS.clone())
    }
}
