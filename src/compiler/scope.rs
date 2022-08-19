//! The scope tree keeps track of nested scopes during compilation.
//! Currently, it's only used resolve jump targets to labels.
use std::collections::{HashMap, VecDeque};

use crate::modules;
use crate::types::ObjectTrait;

pub struct ScopeTree {
    storage: Vec<Scope>,
    pointer: usize,
}

impl ScopeTree {
    pub fn new(addr: usize, initial_scope_kind: ScopeKind) -> Self {
        let global_scope = Scope::new(addr, initial_scope_kind, 0, None);
        Self { storage: vec![global_scope], pointer: 0 }
    }

    pub fn addr(&self) -> usize {
        self.current().addr
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

    pub fn num_locals(&self) -> usize {
        self.current().locals.len()
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
    pub fn add(&mut self, addr: usize, kind: ScopeKind) -> usize {
        let index = self.storage.len();
        self.storage.push(Scope::new(addr, kind, index, Some(self.pointer)));
        self.storage[self.pointer].children.push(index);
        self.pointer = index;
        index
    }

    // Globals ---------------------------------------------------------

    /// Keep track of global vars so we can check to be sure that they
    /// exist when compiling.
    pub fn add_global<S: Into<String>>(&mut self, name: S) {
        if self.in_global_scope() {
            self.current_mut().globals.push(name.into());
        } else {
            panic!("Cannot add global while not in global scope");
        }
    }

    /// See if global exists in or above current scope.
    pub fn has_global(&self, name: &str) -> bool {
        let mut current = self.current();
        loop {
            if current.is_global() {
                return current.globals.iter().any(|n| n == name) || {
                    let builtins = modules::BUILTINS.read().unwrap();
                    builtins.namespace().has(name)
                };
            }
            if let Some(parent_index) = current.parent {
                current = &self.storage[parent_index];
            } else {
                break;
            }
        }
        false
    }

    // Locals ----------------------------------------------------------

    /// Add local var to current scope and return its index within that
    /// scope. If the local already exists in the current scope, just
    /// return its existing stack index.
    pub fn add_local<S: Into<String>>(&mut self, name: S, assigned: bool) -> usize {
        let name = name.into();
        let current = self.current_mut();
        let locals = &mut current.locals;
        if let Some((_, index, _, _)) = locals.iter().find(|(_, _, n, _)| n == &name) {
            *index
        } else {
            let index = locals.len();
            locals.push((current.index, index, name, assigned));
            index
        }
    }

    /// Find local var in current scope or any of its ancestor scopes.
    ///
    /// If the local is found, a tuple with the following fields is
    /// returned: stack index where the local var lives at runtime,
    /// scope pointer of scope where found, local index in scope where
    /// found, assigned flag.
    pub fn find_local(
        &self,
        name: &str,
        pointer: Option<usize>,
    ) -> Option<(usize, usize, usize, bool)> {
        let locals = self.all_locals(pointer);
        if !locals.is_empty() {
            let mut stack_index = locals.len();
            let iter = locals.iter().rev();
            for (pointer, local_index, local_name, assigned) in iter {
                stack_index -= 1;
                if *local_name == name {
                    return Some((stack_index, *pointer, *local_index, *assigned));
                }
            }
        }
        None
    }

    /// Mark local as assigned.
    pub fn mark_assigned(&mut self, pointer: usize, local_index: usize) {
        let scope = self.get_mut(pointer);
        let (pointer, index, name, _) = &scope.locals[local_index];
        scope.locals[local_index] = (*pointer, *index, name.clone(), true);
    }

    /// Flatten locals of scope (current by default) and its ancestors.
    /// The scope's locals will be at the end and the search must
    /// proceed in reverse.
    fn all_locals(&self, pointer: Option<usize>) -> Vec<&(usize, usize, String, bool)> {
        let mut locals = VecDeque::new();
        let mut scope = if let Some(pointer) = pointer {
            self.get(pointer)
        } else {
            self.current()
        };
        loop {
            locals.push_front(&scope.locals);
            if let Some(parent_index) = scope.parent {
                scope = &self.storage[parent_index];
            } else {
                break;
            }
        }
        locals.into_iter().flatten().collect()
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

#[derive(Debug)]
pub struct Scope {
    addr: usize, // address in code block
    kind: ScopeKind,
    index: usize,
    parent: Option<usize>,
    children: Vec<usize>,
    globals: Vec<String>,                      // name
    locals: Vec<(usize, usize, String, bool)>, // scope pointer, index, name, assigned
    /// target label name => jump inst address
    jumps: Vec<(String, usize)>,
    /// label name => label inst address
    labels: HashMap<String, usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScopeKind {
    Module,
    Func,
    Block,
}

impl Scope {
    fn new(addr: usize, kind: ScopeKind, index: usize, parent: Option<usize>) -> Self {
        Self {
            addr,
            kind,
            index,
            parent,
            children: vec![],
            globals: vec![],
            locals: vec![],
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
