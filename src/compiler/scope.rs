//! The scope tree keeps track of nested scopes during compilation.
//! Currently, it's only used resolve jump targets to labels.
use std::collections::{HashMap, VecDeque};

pub struct ScopeTree {
    storage: Vec<Scope>,
    pointer: usize,
}

impl ScopeTree {
    pub fn new() -> Self {
        let global_scope = Scope::new(ScopeKind::Global, 0, None);
        Self { storage: vec![global_scope], pointer: 0 }
    }

    pub fn in_global_scope(&self) -> bool {
        self.current().is_global()
    }

    // Construction ----------------------------------------------------
    //
    // The methods in this section operate on the current scope, which
    // is tracked by a pointer.

    pub fn pointer(&self) -> usize {
        self.pointer
    }

    pub fn current(&self) -> &Scope {
        &self.storage[self.pointer]
    }

    pub fn current_mut(&mut self) -> &mut Scope {
        &mut self.storage[self.pointer]
    }

    pub fn scope_mut(&mut self, index: usize) -> &mut Scope {
        &mut self.storage[index]
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

    fn parent_index(&self) -> Option<usize> {
        match self.pointer {
            0 => None,
            pointer => match self.storage.get(pointer) {
                Some(node) => node.parent,
                None => None,
            },
        }
    }

    // Move up from current scope to its parent (sets current pointer)
    pub fn move_up(&mut self) {
        match self.parent_index() {
            Some(parent_index) => self.pointer = parent_index,
            None => panic!("Could not move up from {}", self.pointer),
        };
    }

    /// Add local var to current scope.
    pub fn add_local<S: Into<String>>(&mut self, name: S) {
        let name = name.into();
        if !self.current().locals.iter().any(|(n, _)| &name == n) {
            self.current_mut().locals.push((name, false));
        }
    }

    /// Find a local var in current scope or any of its ancestor scopes.
    /// The index returned is a pointer into the stack where the local
    /// var lives at runtime. If the local is found, a flag indicating
    /// whether it has been assigned is also returned.
    pub fn find_local<S: Into<String>>(
        &mut self,
        name: S,
        mark_assigned: bool,
    ) -> Option<(usize, bool)> {
        let name = name.into();

        // Flatten locals of current scope and its ancestors. The
        // current scope's locals will be at the end and the search
        // must proceed from the end.
        let mut locals = VecDeque::new();
        let mut scope = self.current();
        let mut count = 0;
        loop {
            let scope_index = scope.index;
            let mut scope_locals = vec![];
            for (local_index, local) in scope.locals.iter().enumerate() {
                scope_locals.push((scope_index, local_index, &local.0, local.1));
                count += 1;
            }
            locals.push_front(scope_locals);
            if let Some(parent_index) = scope.parent {
                scope = &self.storage[parent_index];
            } else {
                break;
            }
        }

        if count == 0 {
            return None;
        }

        let iter = locals.into_iter().flatten().rev();
        let mut index = count;

        for (scope_index, local_index, local_name, assigned) in iter {
            index -= 1;
            if local_name == &name {
                if mark_assigned {
                    let found_scope = self.scope_mut(scope_index);
                    found_scope.locals[local_index] = (name, true);
                }
                return Some((index, assigned));
            }
        }

        None
    }

    // Add jump target and address to current scope
    pub fn add_jump<S: Into<String>>(&mut self, name: S, addr: usize) {
        self.current_mut().jumps.push((name.into(), addr))
    }

    // Add label name and address to current scope
    pub fn add_label<S: Into<String>>(
        &mut self,
        name: S,
        addr: usize,
    ) -> Option<usize> {
        self.current_mut().labels.insert(name.into(), addr)
    }

    // -----------------------------------------------------------------

    fn get(&self, index: usize) -> &Scope {
        &self.storage[index]
    }

    pub fn scope_depth(&self, scope: &Scope) -> usize {
        let mut depth = 0;
        let mut scope = self.get(scope.index);
        while let Some(parent_index) = scope.parent {
            depth += 1;
            scope = self.get(parent_index);
        }
        depth
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
}

#[derive(Debug)]
pub struct Scope {
    pub(crate) kind: ScopeKind,
    index: usize,
    parent: Option<usize>,
    children: Vec<usize>,
    locals: Vec<(String, bool)>, // name, assigned
    /// target label name => jump inst address
    jumps: Vec<(String, usize)>,
    /// label name => label inst address
    labels: HashMap<String, usize>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScopeKind {
    Global,
    Block,
    Func,
}

impl Scope {
    fn new(kind: ScopeKind, index: usize, parent: Option<usize>) -> Self {
        Self {
            kind,
            index,
            parent,
            children: vec![],
            locals: Vec::new(),
            jumps: Vec::new(),
            labels: HashMap::new(),
        }
    }

    fn is_global(&self) -> bool {
        self.kind == ScopeKind::Global
    }

    fn is_leaf(&self) -> bool {
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
