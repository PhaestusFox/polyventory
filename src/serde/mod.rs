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
        let mut ser = ser::InfoSer::new(&mut self.file, reg.deref(), true);
        ser.serialize(object, type_data.type_info())
    }
}

mod tests;
mod error;