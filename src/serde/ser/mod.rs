mod ser_info;
mod ser_struct;
mod ser_opaque;
mod ser_enum;
mod ser_tuple;
mod ser_array;
mod ser_map;

pub use ser_info::InfoSer;

use crate::serde::error::SerdeError;

impl<T: core::fmt::Write> ComponentSer<T> {
    pub fn serialize_reflect(&mut self, object: &dyn bevy::reflect::Reflect) -> Result<(), SerdeError> {
        if object.is_dynamic() {
            return Err(SerdeError::IsDynamic);
        }
        let reg = self.type_repo.read();
        let type_data = reg.get(object.type_id()).ok_or(SerdeError::TypeNotRegistered(object.reflect_type_info().type_path()))?;
        let mut ser = InfoSer::new(&mut self.file, &*reg, true);
        ser.serialize(object, type_data.type_info())
    }
}


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


pub struct ComponentSer<T> {
    pub file: T,
    pub type_repo: bevy::reflect::TypeRegistryArc,
}