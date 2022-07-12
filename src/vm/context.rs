use crate::types::{Builtins, ObjectRef};

use super::constants::Constants;
use super::namespace::Namespace;

pub struct RuntimeContext {
    pub builtins: Builtins,
    constants: Constants,
    // Pointers from var names to constant indexes
    namespace_stack: Vec<Namespace>,
}

impl RuntimeContext {
    pub fn new(
        builtins: Builtins,
        constants: Constants,
        namespace_stack: Vec<Namespace>,
    ) -> Self {
        Self { builtins, constants, namespace_stack }
    }

    pub fn depth(&self) -> usize {
        self.namespace_stack.len() - 1
    }

    pub fn enter_scope(&mut self) {
        self.namespace_stack.push(Namespace::new());
    }

    pub fn exit_scope(&mut self) {
        if self.depth() == 0 {
            panic!("Can't remove global namespace");
        }
        self.namespace_stack.pop();
    }

    // Vars ------------------------------------------------------------

    pub fn declare_var(&mut self, name: &str) {
        let namespace = self.current_namespace();
        // Declaration sets var to nil
        namespace.add(name, 0);
    }

    pub fn assign_var(&mut self, name: &str, const_index: usize) {
        let namespace = self.current_namespace();
        // Assignment points the var at the actual constant value
        namespace.add(name, const_index);
    }

    pub fn add_obj(&mut self, obj: ObjectRef) -> usize {
        self.constants.add(obj)
    }

    pub fn replace_obj(&mut self, index: usize, obj: ObjectRef) {
        self.constants.replace(index, obj)
    }

    pub fn get_obj_index(&self, name: &str) -> Option<&usize> {
        let mut i = self.depth();
        loop {
            let namespace = &self.namespace_stack[i];
            if let Some(usize) = namespace.get(name) {
                break Some(usize);
            }
            if i == 0 {
                break None;
            }
            i -= 1;
        }
    }

    pub fn get_obj(&self, index: usize) -> Option<&ObjectRef> {
        self.constants.get(index)
    }

    pub fn get_obj_by_name(&self, name: &str) -> Option<&ObjectRef> {
        if let Some(&index) = self.get_obj_index(name) {
            self.get_obj(index)
        } else {
            None
        }
    }

    fn current_namespace(&mut self) -> &mut Namespace {
        let index = self.depth();
        &mut self.namespace_stack[index]
    }
}

impl Default for RuntimeContext {
    fn default() -> Self {
        let builtins = Builtins::new();
        let mut constants = Constants::default();
        let mut namespace_stack = vec![];
        let mut label_stack = vec![];
        constants.add(builtins.nil_obj.clone()); // 0
        constants.add(builtins.true_obj.clone()); // 1
        constants.add(builtins.false_obj.clone()); // 2
        namespace_stack.push(Namespace::new());
        label_stack.push(Namespace::new());
        RuntimeContext::new(builtins, constants, namespace_stack)
    }
}
