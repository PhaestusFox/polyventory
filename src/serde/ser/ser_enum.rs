use bevy::reflect::{EnumInfo, GetPath, NamedField, OffsetAccess, ParsedPath, PartialReflect, Reflect, ReflectRef, ReflectSerialize, UnnamedField};

use crate::serde::error::SerdeError;

impl<T: core::fmt::Write> super::InfoSer<'_, T> {
    pub fn serialise_enum(&mut self, v: &dyn Reflect, info: &EnumInfo) -> Result<(), SerdeError> {
        if self.name(info)? {
            write!(self.file, ": ")?;
        }
        let ReflectRef::Enum(data) = v.reflect_ref() else {
            todo!("Todo reflect passed in to serde enum is not enum");
        };
        write!(self.file, "{}", data.variant_name())?;
        match info.variant_at(data.variant_index()).expect("This variant exists") {
            bevy::reflect::VariantInfo::Struct(struct_variant_info) => {
                self.serialise_namedfields(v, struct_variant_info.iter())
            },
            bevy::reflect::VariantInfo::Tuple(tuple_variant_info) => {
                self.serialise_fields(v, tuple_variant_info.iter())
            },
            bevy::reflect::VariantInfo::Unit(_) => Ok(()),
        }
    }

    pub fn serialise_namedfields<'a>(&mut self, v: &dyn Reflect, fields: impl Iterator<Item = &'a NamedField>) -> Result<(), SerdeError> {
        write!(self.file, "{{")?;
        
        for (i, f) in fields.enumerate() {
            if i != 0 {
                write!(self.file, ",")?;
            }
            let info = f.type_info().ok_or(SerdeError::TypeNotRegistered(f.type_path()))?;
            let v = v.reflect_path(&ParsedPath(vec![OffsetAccess {
                access: bevy::reflect::Access::FieldIndex(i),
                offset: None
            }]))?;
            let v = v.try_as_reflect().ok_or(SerdeError::NotFullReflect(info.type_path()))?;
            write!(self.file, "\n{}: ", f.name())?;
            self.serialize(v, info)?;
        }
        write!(self.file, "\n}}").map_err(SerdeError::WrightError)
    }
}