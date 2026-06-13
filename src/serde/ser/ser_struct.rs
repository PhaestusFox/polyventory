use bevy::reflect::{GetPath, OffsetAccess, ParsedPath, Reflect, StructInfo, TupleStructInfo, TypeRegistry};

use crate::serde::error::SerdeError;

impl<T: core::fmt::Write> super::InfoSer<'_, T> {
    pub fn serialise_struct(&mut self, v: &dyn Reflect, info: &StructInfo) -> Result<(), SerdeError> {
        if info.field_len() == 0 {
            // if it has not fields it is a marker so we just put its name
            self.named = true;
            self.name(info)?;
            return Ok(());
        }
        self.name(info)?;
        self.serialise_namedfields(v, info.iter())
    }

    pub fn serialize_tuple_struct(&mut self, v: &dyn Reflect, info: &TupleStructInfo) -> Result<(), SerdeError> {
        self.name(info)?;
        self.serialise_fields(v, info.iter())
    }
}