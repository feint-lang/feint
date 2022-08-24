use std::slice;

use indexmap;

use crate::modules;
use crate::types::{new, Namespace, ObjectRef, ObjectTrait};
use crate::vm::RuntimeObjResult;

use super::constants::Constants;
use super::result::{RuntimeErr, RuntimeResult};

pub struct RuntimeContext {
    global_constants: Constants,
    ns_stack: Vec<Namespace>,
    // Number of namespaces on stack
    size: usize,
    // Index of namespace associated with current scope
    current_depth: usize,
}

impl RuntimeContext {
    pub fn new() -> Self {
        let mut ctx = Self {
            global_constants: Constants::new(),
            ns_stack: vec![],
            size: 0,
            current_depth: 0,
        };
        ctx.init();
        ctx
    }

    fn init(&mut self) {
        // Add singleton constants.
        self.add_global_const(new::nil()); // 0
        self.add_global_const(new::true_()); // 1
        self.add_global_const(new::false_()); // 2

        for int in new::SHARED_INTS.iter() {
            self.add_global_const(int.clone());
        }

        // Enter global scope.
        self.enter_scope();
    }

    #[inline]
    fn current_ns(&self) -> &Namespace {
        &self.ns_stack[self.current_depth]
    }

    fn current_ns_mut(&mut self) -> &mut Namespace {
        &mut self.ns_stack[self.current_depth]
    }

    pub fn enter_scope(&mut self) {
        self.current_depth = self.size;
        self.ns_stack.push(Namespace::new());
        self.size += 1;
    }

    pub fn exit_scope(&mut self) {
        if self.current_depth == 0 {
            panic!("Can't remove global namespace");
        }
        let mut ns = self.ns_stack.pop().expect("Expected namespace");
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
        starting_depth: Option<usize>,
    ) -> Result<usize, RuntimeErr> {
        let ns_stack = &self.ns_stack;
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
    pub fn get_var(&self, name: &str) -> RuntimeObjResult {
        let depth = self.get_var_depth(name, None)?;
        self.get_var_at_depth(depth, name)
    }

    /// Get var in parent namespace or any ancestor of the parent
    /// namespace.
    pub fn get_outer_var(&self, name: &str) -> RuntimeObjResult {
        let depth = self.get_outer_var_depth(name)?;
        self.get_var_at_depth(depth, name)
    }

    /// Get builtin object. This is used as a fallback when a name isn't
    /// found in the current scope.
    /// TODO: Cache builtins up front (like before, just not in the
    ///       global namespace).
    pub fn get_builtin(&self, name: &str) -> RuntimeObjResult {
        let builtins = modules::BUILTINS.read().unwrap();
        let builtins = builtins.down_to_mod().unwrap();
        if let Some(obj) = builtins.ns().get_obj(name) {
            Ok(obj)
        } else {
            let message = format!("Name not found: {name}");
            Err(RuntimeErr::name_err(message))
        }
    }

    pub fn iter_vars(&self) -> indexmap::map::Iter<'_, String, ObjectRef> {
        self.ns_stack[self.current_depth].iter()
    }
}
