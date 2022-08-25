//! The scope tree keeps track of nested scopes during compilation.
//! Currently, it's only used resolve jump targets to labels.
use std::collections::HashMap;

pub struct ScopeTree {
    storage: Vec<Scope>,
    pointer: usize,
}

impl ScopeTree {
    pub fn new(initial_scope_kind: ScopeKind) -> Self {
        let global_scope = Scope::new(initial_scope_kind, 0, None);
        Self { storage: vec![global_scope], pointer: 0 }
    }

    pub fn pointer(&self) -> usize {
        self.pointer
    }

    pub fn in_global_scope(&self) -> bool {
        self.current().is_global()
    }

    pub fn in_func_scope(&self) -> bool {
        self.current().is_func()
    }

    pub fn _in_block_scope(&self) -> bool {
        self.current()._is_block()
    }

    fn get(&self, index: usize) -> &Scope {
        &self.storage[index]
    }

    fn get_mut(&mut self, index: usize) -> &mut Scope {
        &mut self.storage[index]
    }

    // Traversal -------------------------------------------------------

    fn parent_index(&self) -> Option<usize> {
        match self.pointer {
            0 => None,
            pointer => match self.storage.get(pointer) {
                Some(node) => node.parent,
                None => None,
            },
        }
    }

    fn scope_depth(&self, scope: &Scope) -> usize {
        let mut depth = 0;
        let mut scope = self.get(scope.index);
        while let Some(parent_index) = scope.parent {
            depth += 1;
            scope = self.get(parent_index);
        }
        depth
    }

    /// Move up from current scope to its parent (sets current pointer).
    pub fn move_up(&mut self) {
        match self.parent_index() {
            Some(parent_index) => self.pointer = parent_index,
            None => panic!("Could not move up from {}", self.pointer),
        };
    }

    /// For each leaf scope, apply the specified visit function to the
    /// leaf scope first and then to each of its parent scopes in turn.
    /// Note that parent scopes will be processed multiple times when a
    /// parent scope contains multiple nested scopes. The visit function
    /// will be passed the current scope and its depth.
    pub fn walk_up(&self, mut visit: impl FnMut(&Scope, usize) -> bool) {
        for scope in self.storage.iter().filter(|n| n.is_leaf()) {
            let depth = self.scope_depth(scope);
            if visit(scope, depth) {
                match scope.parent {
                    Some(parent_index) => {
                        let parent_scope = self.get(parent_index);
                        if !visit(parent_scope, depth - 1) {
                            break;
                        }
                    }
                    _ => break,
                }
            } else {
                break;
            }
        }
    }

    // Current Scope ---------------------------------------------------
    //
    // NOTE: All methods from here to the end of the impl operate on the
    //       current scope.

    fn current(&self) -> &Scope {
        &self.storage[self.pointer]
    }

    fn current_mut(&mut self) -> &mut Scope {
        &mut self.storage[self.pointer]
    }

    /// Add nested scope to current scope then make the new scope the
    /// current scope.
    pub fn add(&mut self, kind: ScopeKind) -> usize {
        let index = self.storage.len();
        self.storage.push(Scope::new(kind, index, Some(self.pointer)));
        self.storage[self.pointer].children.push(index);
        self.pointer = index;
        index
    }

    // Vars ------------------------------------------------------------

    /// Add var to *current* scope if it's not already present.
    pub fn add_var<S: Into<String>>(&mut self, addr: usize, name: S, assigned: bool) {
        let name = name.into();
        let pointer = self.pointer;
        if !self.get(pointer).vars.iter().any(|v| v.name == name) {
            self.current_mut().vars.push(Var { addr, pointer, name, assigned });
        }
    }

    /// Mark var in scope with name as assigned. Note that var *must*
    /// exist in the specified scope or this will panic.
    pub fn mark_assigned(&mut self, pointer: usize, name: &str) {
        let scope = self.get_mut(pointer);
        let result =
            scope.vars.iter().position(|v| v.pointer == pointer && v.name == name);
        if let Some(index) = result {
            let var = &scope.vars[index];
            let mut new_var = var.clone();
            new_var.assigned = true;
            scope.vars[index] = new_var;
        } else {
            panic!("Var does not exist in scope {pointer}: {name}")
        }
    }

    /// Find var in current scope or any of its ancestor scopes.
    pub fn find_var(&self, name: &str, pointer: Option<usize>) -> Option<Var> {
        let mut scope = if let Some(pointer) = pointer {
            self.get(pointer)
        } else {
            self.current()
        };
        loop {
            if let Some(var) = scope.vars.iter().find(|v| v.name == name) {
                return Some(var.clone());
            }
            if let Some(parent_index) = scope.parent {
                scope = self.get(parent_index)
            } else {
                break;
            }
        }
        None
    }

    /// Find var in parent scope or any of its ancestor scopes.
    pub fn find_var_in_parent(&self, name: &str) -> Option<Var> {
        if self.pointer == 0 {
            None
        } else {
            self.find_var(name, Some(self.pointer - 1))
        }
    }

    // Jumps & Labels --------------------------------------------------

    /// Add jump target and address to current scope
    pub fn add_jump<S: Into<String>>(&mut self, name: S, addr: usize) {
        self.current_mut().jumps.push((name.into(), addr))
    }

    /// Add label name and address to current scope
    pub fn add_label<S: Into<String>>(
        &mut self,
        name: S,
        addr: usize,
    ) -> Option<usize> {
        self.current_mut().labels.insert(name.into(), addr)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScopeKind {
    Module,
    Func,
    Block,
}

#[derive(Clone, Debug)]
pub struct Var {
    pub addr: usize,
    pub pointer: usize,
    pub name: String,
    pub assigned: bool,
}

#[derive(Debug)]
pub struct Scope {
    kind: ScopeKind,
    index: usize,
    parent: Option<usize>,
    children: Vec<usize>,
    vars: Vec<Var>,
    /// target label name => jump inst address
    jumps: Vec<(String, usize)>,
    /// label name => label inst address
    labels: HashMap<String, usize>,
}

impl Scope {
    fn new(kind: ScopeKind, index: usize, parent: Option<usize>) -> Self {
        Self {
            kind,
            index,
            parent,
            children: vec![],
            vars: vec![],
            jumps: vec![],
            labels: HashMap::new(),
        }
    }

    pub fn is_global(&self) -> bool {
        self.kind == ScopeKind::Module
    }

    pub fn is_func(&self) -> bool {
        self.kind == ScopeKind::Func
    }

    pub fn _is_block(&self) -> bool {
        self.kind == ScopeKind::Block
    }

    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    pub fn jumps(&self) -> &Vec<(String, usize)> {
        &self.jumps
    }

    /// Find label for jump target in this scope or its parent scopes.
    /// When a target label is found, its instruction address and scope
    /// depth are returned. Otherwise, None is returned.
    pub fn find_label(
        &self,
        tree: &ScopeTree,
        name: &str,
        jump_addr: Option<&usize>,
    ) -> Option<(usize, usize)> {
        let jump_addr = match jump_addr {
            Some(addr) => addr,
            None => {
                if let Some(pos) = self.jumps.iter().position(|(n, _)| n == name) {
                    &self.jumps[pos].1
                } else {
                    panic!("Jump does not exist in scope: {}", name)
                }
            }
        };

        if let Some(label_addr) = self.labels.get(name) {
            if label_addr > jump_addr {
                return Some((*label_addr, tree.scope_depth(self)));
            }
        }

        match self.parent {
            Some(parent_index) => {
                let parent_scope = tree.get(parent_index);
                parent_scope.find_label(tree, name, Some(jump_addr))
            }
            None => None,
        }
    }
}
