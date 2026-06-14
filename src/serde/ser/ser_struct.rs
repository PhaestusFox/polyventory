use bevy::reflect::{GetPath, OffsetAccess, ParsedPath, Reflect, StructInfo, TupleStructInfo, TypeRegistry};

use crate::serde::error::SerdeError;

impl<T: core::fmt::Write> super::InfoSer<'_, T> {
    pub fn serialise_struct(&mut self, v: &dyn Reflect, info: &StructInfo) -> Result<(), SerdeError> {
        self.name(info);
        if info.field_len() == 0 {
            return Ok(());
        }
        self.serialise_namedfields(v, info.iter())
    }

    pub fn serialize_tuple_struct(&mut self, v: &dyn Reflect, info: &TupleStructInfo) -> Result<(), SerdeError> {
        self.name(info);
        self.serialise_fields(v, info.iter())
    }
}