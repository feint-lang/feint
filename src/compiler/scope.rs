use std::collections::HashMap;

pub struct ScopeTree {
    storage: Vec<Scope>,
    pointer: usize,
}

impl ScopeTree {
    pub fn new() -> Self {
        Self { storage: vec![Scope::new(0, None)], pointer: 0 }
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

    // Add child
    pub fn add(&mut self) -> usize {
        let index = self.storage.len();
        self.storage.push(Scope::new(index, Some(self.pointer)));
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

    // Get jumps for the specified scope and all of its nested scopes.
    pub fn all_jumps_for_scope(&self, index: usize) -> Vec<HashMap<String, usize>> {
        let mut jumps = vec![];
        let scope = self.get(index);
        if scope.jumps.len() > 0 {
            jumps.push(scope.jumps.clone());
        }
        for child_index in scope.children.iter() {
            jumps.extend(self.all_jumps_for_scope(*child_index));
        }
        jumps
    }

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
    index: usize,
    parent: Option<usize>,
    children: Vec<usize>,
    labels: HashMap<String, usize>,
    jumps: HashMap<String, usize>,
}

impl Scope {
    fn new(index: usize, parent: Option<usize>) -> Self {
        Self {
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

    pub fn jumps(&self) -> &HashMap<String, usize> {
        &self.jumps
    }

    pub fn labels(&self) -> &HashMap<String, usize> {
        &self.labels
    }

    fn is_leaf(&self) -> bool {
        self.children.len() == 0
    }
}
