use std::collections::HashMap;

use rand::Rng;

#[derive(Debug)]
pub struct Type<'a> {
    pub id: usize,
    pub name: &'a str,
    pub methods: HashMap<&'a str, &'a Method>,
}

impl<'a> Type<'a> {
    pub fn new(name: &'a str, methods: HashMap<&'a str, &'a Method>) -> Self {
        let mut rng = rand::thread_rng();
        let id: usize = rng.gen();
        Type { id, name, methods }
    }

    pub fn new_instance(
        &self,
        name: String,
        attributes: HashMap<&'a str, &'a Object>,
    ) -> Object {
        let mut rng = rand::thread_rng();
        let id: usize = rng.gen();
        Object { id, name, type_: &self, attributes }
    }
}

#[derive(Debug)]
pub struct Object<'a> {
    pub id: usize,
    pub name: String,
    pub type_: &'a Type<'a>,
    pub attributes: HashMap<&'a str, &'a Object<'a>>,
}

#[derive(Debug)]
pub struct Method {
    name: String,
    // ???
}

pub struct IntType {}
