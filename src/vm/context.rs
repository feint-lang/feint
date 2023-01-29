//! VM runtime context.
use indexmap::IndexMap;

use crate::modules::std::STD;
use crate::types::{new, ObjectRef, ObjectTrait};
use crate::vm::RuntimeObjResult;

use super::result::{RuntimeErr, RuntimeResult};

type Namespace = IndexMap<String, ObjectRef>;
type NamespaceStack = Vec<Namespace>;

/// Holds info relating to execution of the current module.
///
/// Currently, this is just the stack of namespaces corresponding to
/// scopes as they are entered and exited.
///
/// At the bottom is the namespace corresponding to the module's global
/// scope. This namespace cannot be popped from the stack--trying to
/// do so will cause a `panic`.
///
/// At the top is the namespace corresponding to the current scope. A
/// new namespace is pushed every time the VM encounters a `SCOPE_START`
/// instruction and popped every time the VM encounters a `SCOPE_END`
/// instruction.
pub struct ModuleExecutionContext {
    ns_stack: NamespaceStack,
}

impl Default for ModuleExecutionContext {
    fn default() -> Self {
        Self { ns_stack: vec![IndexMap::default()] }
    }
}

impl ModuleExecutionContext {
    /// This will panic if the builtin doesn't exist because builtin
    /// names are resolved during compilation.
    pub(super) fn get_builtin(&self, name: &str) -> ObjectRef {
        let std = STD.read().unwrap();
        let std = std.down_to_mod().unwrap();
        std.get_global(name).expect("Builtin unexpectedly undefined: {name}")
    }

    #[inline]
    pub(crate) fn globals(&self) -> &Namespace {
        &self.ns_stack[0]
    }

    pub(super) fn get_global(&self, name: &str) -> Option<ObjectRef> {
        self.globals().get(name).cloned()
    }

    pub(super) fn enter_scope(&mut self) {
        self.ns_stack.push(IndexMap::default());
    }

    pub(super) fn exit_scope(&mut self) {
        if self.current_depth() == 0 {
            panic!("Global namespace cannot be exited");
        }
        let mut ns = self.ns_stack.pop().expect("Expected namespace");
        ns.clear();
    }

    pub(super) fn exit_all_scopes(&mut self) {
        while self.ns_stack.len() > 1 {
            self.exit_scope();
        }
    }

    pub(super) fn reset(&mut self) {
        self.exit_all_scopes();
        self.ns_stack[0].clear();
    }

    #[inline]
    fn current(&self) -> &Namespace {
        self.ns_stack.last().unwrap()
    }

    #[inline]
    fn current_mut(&mut self) -> &mut Namespace {
        self.ns_stack.last_mut().unwrap()
    }

    #[inline]
    fn current_depth(&self) -> usize {
        self.ns_stack.len() - 1
    }

    // Vars ------------------------------------------------------------

    /// Declare a new var in the current namespace. This adds a slot for
    /// the var in the current namespace and sets its initial value to
    /// nil.
    pub(super) fn declare_var(&mut self, name: &str) {
        let initial = new::nil();
        let ns = self.current_mut();
        ns.insert(name.to_owned(), initial);
    }

    /// Assign value to var in *current* namespace. This looks up the
    /// var by name in the current namespace, updates its value, and
    /// returns the depth of the namespace where the var lives. If the
    /// var doesn't exist in the current namespace, an error is returned
    /// instead (indicating an internal error).
    pub(super) fn assign_var(
        &mut self,
        name: &str,
        obj: ObjectRef,
    ) -> Result<usize, RuntimeErr> {
        let ns = self.current_mut();
        if ns.contains_key(name) {
            ns.insert(name.to_owned(), obj);
            Ok(self.ns_stack.len() - 1)
        } else {
            let message = format!("Name not defined in current scope: {name}");
            Err(RuntimeErr::name_err(message))
        }
    }

    /// Conveniently declare and assign a var in one step.
    pub(super) fn declare_and_assign_var(
        &mut self,
        name: &str,
        obj: ObjectRef,
    ) -> Result<usize, RuntimeErr> {
        self.declare_var(name);
        self.assign_var(name, obj)
    }

    /// Assign value to var--reach into the scope at depth and set the
    /// var at the specified index.
    pub(super) fn assign_var_at_depth(
        &mut self,
        depth: usize,
        name: &str,
        obj: ObjectRef,
    ) -> RuntimeResult {
        let ns = &mut self.ns_stack[depth];
        if ns.contains_key(name) {
            ns.insert(name.to_owned(), obj);
            Ok(())
        } else {
            let message = format!("Name not defined at depth {depth}: {name}");
            Err(RuntimeErr::name_err(message))
        }
    }

    /// Get depth of scope where var is defined.
    pub(super) fn get_var_depth(
        &self,
        name: &str,
        offset: usize,
    ) -> Result<usize, RuntimeErr> {
        let ns_stack = &self.ns_stack;
        let mut var_depth = self.current_depth() - offset;
        loop {
            if ns_stack[var_depth].get(name).is_some() {
                break Ok(var_depth);
            }
            if var_depth == 0 {
                let message = format!("Name not found: {name}");
                break Err(RuntimeErr::name_err(message));
            }
            var_depth -= 1;
        }
    }

    /// Get var from current scope.
    pub(super) fn get_var_in_current_ns(&self, name: &str) -> RuntimeObjResult {
        let ns = self.current();
        if let Some(obj) = ns.get(name) {
            Ok(obj.clone())
        } else {
            let message = format!("Name not defined in current scope: {name}");
            Err(RuntimeErr::name_err(message))
        }
    }

    /// Reach into the scope at depth and get the specified var.
    pub(super) fn get_var_at_depth(
        &self,
        depth: usize,
        name: &str,
    ) -> RuntimeObjResult {
        if let Some(obj) = self.ns_stack[depth].get(name) {
            Ok(obj.clone())
        } else {
            let message = format!("Name not defined at depth {depth}: {name}");
            Err(RuntimeErr::name_err(message))
        }
    }

    /// Get var in current scope or any ancestor scope.
    pub(super) fn get_var(&self, name: &str, offset: usize) -> RuntimeObjResult {
        let depth = self.get_var_depth(name, offset)?;
        self.get_var_at_depth(depth, name)
    }
}
