use rand::Rng;
use std::collections::HashMap;
use std::fmt;

type Name = &'static str;
type Slots = HashMap<Name, Name>;

pub struct Type {
    pub id: u128,
    pub name: Name,
    pub slots: Slots,
}

impl Type {
    // Create a new type
    pub fn create(
        name: Name,
        slots: Slots,
    ) -> Type {
        let mut rng = rand::thread_rng();
        let id: u128 = rng.gen();
        Type { id, name, slots }
    }

    // Create a new instance of this type
    pub fn new(self) -> Instance {
        Instance { class: self }
    }
}

pub struct Instance {
    pub class: Type,
}

impl Instance {
    pub fn to_string(&self) -> String {
        format!("{}: ", self.class.name)
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
