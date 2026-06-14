use std::io::{Cursor, Read};

use bevy::reflect::{Reflect, TypeRegistry, TypeRegistryArc};

mod de_opaque;

pub struct ComponentDe {
    pub type_repo: TypeRegistryArc,
}

impl ComponentDe {
    pub fn deserialize_str(&self, data: &str) -> Box<dyn Reflect> {
        let mut c = Cursor::new(data);
        let r = self.type_repo.read();
        let mut de = InfoDe {
            file: &mut c,
            registry: &r,
        };
        de.deserialize()
    }
}

pub struct InfoDe<'a, T> {
    pub(super) file: &'a mut Cursor<T>,
    pub(super) registry: &'a TypeRegistry,
}

impl<T: AsRef<str>> InfoDe<'_, T> {
    pub fn deserialize(&mut self) -> Box<dyn Reflect> {
        // read the begging of the file to work out what struct we are trying to read
        todo!()
    }
}