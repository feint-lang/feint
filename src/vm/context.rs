use crate::types::Builtins;

use super::constants::Constants;
use super::namespace::Namespace;

pub struct RuntimeContext {
    pub builtins: Builtins,
    pub constants: Constants,
    pub namespace_stack: Vec<Namespace>,
}

impl RuntimeContext {
    pub fn new(
        builtins: Builtins,
        constants: Constants,
        namespace_stack: Vec<Namespace>,
    ) -> Self {
        Self { builtins, constants, namespace_stack }
    }

    pub fn add_namespace(&mut self) {
        self.namespace_stack.push(Namespace::new());
    }

    pub fn pop_namespace(&mut self) -> Namespace {
        if self.namespace_stack.len() == 1 {
            panic!("Can't remove global namespace");
        }
        self.namespace_stack.pop().unwrap()
    }

    pub fn current_namespace(&self) -> &Namespace {
        let index = self.namespace_stack.len() - 1;
        &self.namespace_stack[index]
    }

    fn current_namespace_mut(&mut self) -> &mut Namespace {
        let index = self.namespace_stack.len() - 1;
        &mut self.namespace_stack[index]
    }

    pub fn declare_var(&mut self, name: &str) {
        let namespace = self.current_namespace_mut();
        // Declaration sets var to nil
        namespace.add(name, 0);
    }

    pub fn assign_var(&mut self, name: &str, const_index: usize) {
        let namespace = self.current_namespace_mut();
        // Assignment points the var at the actual constant value
        namespace.add(name, const_index);
    }

    pub fn get_var(&self, name: &str) -> Option<&usize> {
        let mut i = self.namespace_stack.len() - 1;
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
}

impl Default for RuntimeContext {
    fn default() -> Self {
        let builtins = Builtins::new();
        let mut constants = Constants::default();
        let mut namespace_stack = vec![];
        constants.add(builtins.nil_obj.clone()); // 0
        constants.add(builtins.true_obj.clone()); // 1
        constants.add(builtins.false_obj.clone()); // 2
        namespace_stack.push(Namespace::new());
        RuntimeContext::new(builtins, constants, namespace_stack)
    }
}
