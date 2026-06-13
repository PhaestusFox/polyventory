use bevy::reflect::{PartialReflect, Reflect, TypeInfo, TypeRegistry};

use crate::serde::error::SerdeError;

pub struct InfoSer<'a, T> {
    pub(super) file: &'a mut T,
    pub(super) named: bool,
    pub(super) registry: &'a TypeRegistry,
}

impl<'a, T> InfoSer<'a, T> {
    pub fn new(file: &'a mut T, registry: &'a TypeRegistry, named: bool) -> Self {
        Self { file, registry, named }
    }
}

impl<T: core::fmt::Write> InfoSer<'_, T> {
    pub fn serialize(&mut self, v: &dyn Reflect, info: &TypeInfo) -> Result<(), SerdeError> {
        match info {
            bevy::reflect::TypeInfo::Struct(struct_info) => self.serialise_struct(v, struct_info),
            bevy::reflect::TypeInfo::TupleStruct(tuple_struct_info) => self.serialize_tuple(v, tuple_struct_info),
            bevy::reflect::TypeInfo::Tuple(tuple_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::List(list_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::Array(array_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::Map(map_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::Set(set_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::Enum(enum_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::Opaque(opaque_info) => self.serialise_opaque(v, opaque_info),
        }
    }

}