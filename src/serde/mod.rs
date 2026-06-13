use std::ops::Deref;

use bevy::reflect::{Reflect, StructInfo, TypeRegistry, TypeRegistryArc};

use crate::serde::error::SerdeError;

pub struct ComponentDeSer;

pub struct ComponentSer<T> {
    file: T,
    type_repo: TypeRegistryArc,
}

mod ser;

/*
impl serde::Serializer for ComponentSer {
    type Ok = ();
    type Error = error::SerdeError;

    type SerializeMap;
    
    type SerializeSeq;
    
    type SerializeTuple;
    
    type SerializeTupleStruct;
    
    type SerializeTupleVariant;
    
    type SerializeStruct;
    
    type SerializeStructVariant;
}
    */

impl<T: core::fmt::Write> ComponentSer<T> {
    fn serialize_reflect(&mut self, object: &dyn Reflect) -> Result<(), SerdeError> {
        if object.is_dynamic() {
            return Err(SerdeError::IsDynamic);
        }
        let reg = self.type_repo.read();
        let type_data = reg.get(object.type_id()).ok_or(SerdeError::TypeNotRegistered(object.reflect_type_info().type_path()))?;
        match type_data.type_info() {
            bevy::reflect::TypeInfo::Struct(struct_info) => ser::StructSer::new(&mut self.file, reg.deref(), true).serialise_struct(struct_info),
            bevy::reflect::TypeInfo::TupleStruct(tuple_struct_info) => ser::StructSer::new(&mut self.file, reg.deref(), true).serialize_tuple(tuple_struct_info),
            bevy::reflect::TypeInfo::Tuple(tuple_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::List(list_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::Array(array_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::Map(map_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::Set(set_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::Enum(enum_info) => unimplemented!("serialize reflect"),
            bevy::reflect::TypeInfo::Opaque(opaque_info) => unimplemented!("serialize reflect"),
        }
    }
}

mod tests;
mod error;