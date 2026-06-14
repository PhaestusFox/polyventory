use bevy::reflect::{PartialReflect, Reflect, TypeInfo, TypeRegistry};

use crate::serde::{error::SerdeError, naming::HasName};

pub struct InfoSer<'a, T> {
    pub(super) file: &'a mut T,
    pub(super) named: bool,
    pub(super) registry: &'a TypeRegistry,
}

impl<'a, T> InfoSer<'a, T> {
    pub fn new(file: &'a mut T, registry: &'a TypeRegistry) -> Self {
        Self { file, registry, named: true }
    }
}

impl<T: core::fmt::Write> InfoSer<'_, T> {
    pub fn serialize(&mut self, v: &dyn Reflect, info: &TypeInfo) -> Result<(), SerdeError> {
        match info {
            bevy::reflect::TypeInfo::Struct(struct_info) => self.serialise_struct(v, struct_info),
            bevy::reflect::TypeInfo::TupleStruct(tuple_struct_info) => self.serialize_tuple_struct(v, tuple_struct_info),
            bevy::reflect::TypeInfo::Tuple(tuple_info) => self.serialise_tuple(v, tuple_info),
            bevy::reflect::TypeInfo::List(list_info) => self.serialise_list(v, list_info),
            bevy::reflect::TypeInfo::Array(array_info) => self.serialise_array(v, array_info),
            bevy::reflect::TypeInfo::Map(map_info) => self.serialize_map(v, map_info),
            bevy::reflect::TypeInfo::Set(set_info) => self.serialize_set(v, set_info),
            bevy::reflect::TypeInfo::Enum(enum_info) => self.serialise_enum(v, enum_info),
            bevy::reflect::TypeInfo::Opaque(opaque_info) => self.serialise_opaque(v, opaque_info),
        }
    }

    pub fn name(&mut self, name: impl HasName) -> Result<bool, SerdeError> {
        if self.named {
            self.named = false;
            write!(self.file, "{}", name.name())?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}