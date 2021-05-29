use crate::types::Builtins;

use super::constants::Constants;
use super::namespace::Namespace;

pub struct RuntimeContext {
    pub builtins: Builtins,
    pub constants: Constants,
    // Pointers from var names to constant indexes
    pub namespace_stack: Vec<Namespace>,
    // Pointers from label names to instruction indexes
    pub label_stack: Vec<Namespace>,
}

impl RuntimeContext {
    pub fn new(
        builtins: Builtins,
        constants: Constants,
        namespace_stack: Vec<Namespace>,
        label_stack: Vec<Namespace>,
    ) -> Self {
        Self { builtins, constants, namespace_stack, label_stack }
    }

    pub fn block_depth(&self) -> usize {
        self.namespace_stack.len()
    }

    pub fn enter_block(&mut self) {
        self.namespace_stack.push(Namespace::new());
        self.label_stack.push(Namespace::new());
    }

    pub fn exit_block(&mut self) {
        if self.namespace_stack.len() == 1 {
            panic!("Can't remove global namespace");
        }
        self.namespace_stack.pop();
        self.label_stack.pop();
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

    fn current_namespace(&mut self) -> &mut Namespace {
        let index = self.namespace_stack.len() - 1;
        &mut self.namespace_stack[index]
    }

    // Labels ----------------------------------------------------------

    pub fn add_label(&mut self, name: &str, ip: usize) {
        let label_space = self.current_label_space();
        label_space.add(name, ip);
    }

    pub fn get_label(&mut self, name: &str) -> Option<&usize> {
        let label_space = self.current_label_space();
        label_space.get(name)
    }

    fn current_label_space(&mut self) -> &mut Namespace {
        let index = self.label_stack.len() - 1;
        &mut self.label_stack[index]
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
        RuntimeContext::new(builtins, constants, namespace_stack, label_stack)
    }
}
