//! The scope tree keeps track of nested scopes during compilation.
//! Currently, it's only used resolve jump targets to labels.
use std::collections::HashMap;

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

    // Add jump target and address to current scope
    pub fn add_jump(&mut self, name: &str, addr: usize) -> Option<usize> {
        self.current_mut().jumps.insert(name.to_owned(), addr)
    }

    // Add label name and address to current scope
    pub fn add_label(&mut self, name: &str, addr: usize) -> Option<usize> {
        self.current_mut().labels.insert(name.to_owned(), addr)
    }

    // -----------------------------------------------------------------

    fn get(&self, index: usize) -> &Scope {
        &self.storage[index]
    }

    fn scope_depth(&self, index: usize) -> usize {
        let mut depth = 0;
        let mut scope = self.get(index);
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
            let depth = self.scope_depth(scope.index);
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
    kind: ScopeKind,
    index: usize,
    parent: Option<usize>,
    children: Vec<usize>,
    /// label name => label inst address
    labels: HashMap<String, usize>,
    /// target label name => jump inst address
    jumps: HashMap<String, usize>,
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
            labels: HashMap::new(),
            jumps: HashMap::new(),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    fn is_global(&self) -> bool {
        self.kind == ScopeKind::Global
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }

    pub fn jumps(&self) -> &HashMap<String, usize> {
        &self.jumps
    }

    pub fn labels(&self) -> &HashMap<String, usize> {
        &self.labels
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
            None => match self.jumps.get(name) {
                Some(addr) => addr,
                None => panic!("Jump does not exist in scope: {}", name),
            },
        };

        if let Some(label_addr) = self.labels.get(name) {
            if label_addr > jump_addr {
                return Some((*label_addr, tree.scope_depth(self.index)));
            }
        }

        // Disallow jump out of function
        if let ScopeKind::Func = self.kind {
            return None;
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
