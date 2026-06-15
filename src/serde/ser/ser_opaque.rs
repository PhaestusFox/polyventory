use bevy::reflect::{OpaqueInfo, PartialReflect, Reflect, ReflectSerialize};

use crate::serde::error::SerdeError;

impl<T: core::fmt::Write> super::InfoSer<'_, T> {
    pub fn serialise_opaque(&mut self, v: &dyn Reflect, info: &OpaqueInfo) -> Result<(), SerdeError> {
        // if we get here and print the name of an opaque type we need to add a : to split the value
        if self.name(info)? {
            write!(self.file, ": ")?;
        }
        let reg = self.registry.get_type_data::<ReflectSerialize>(info.type_id()).ok_or(SerdeError::OpaqueNotSerde(info.type_path()))?;
        let mut ser = ron::Serializer::new(&mut *self.file, None).expect("To Work");
        reg.serialize(v, &mut ser).expect("ron to work");
        Ok(())
    }
}