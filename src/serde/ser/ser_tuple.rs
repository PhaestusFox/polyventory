use bevy::reflect::{EnumInfo, GetPath, NamedField, OffsetAccess, ParsedPath, PartialReflect, Reflect, ReflectRef, ReflectSerialize, TupleInfo, UnnamedField};

use crate::serde::error::SerdeError;

impl<T: core::fmt::Write> super::InfoSer<'_, T> {
    pub fn serialise_tuple(&mut self, v: &dyn Reflect, info: &TupleInfo) -> Result<(), SerdeError> {
        self.name(info);
        self.serialise_fields(v, info.iter())
    }
    
    pub fn serialise_fields<'a>(&mut self, v: &dyn Reflect, fields: impl Iterator<Item = &'a UnnamedField>) -> Result<(), SerdeError> {
        write!(self.file, "(")?;
        let trail = fields.size_hint().0 != 0;
        for (i, f) in fields.enumerate() {
            if i != 0 {
                write!(self.file, ",\n")?;
            } else {
                write!(self.file, "\n")?;
            }
            let info = f.type_info().ok_or(SerdeError::TypeNotRegistered(f.type_path()))?;
            let v = v.reflect_path(&ParsedPath(vec![OffsetAccess {
                access: bevy::reflect::Access::TupleIndex(f.index()),
                offset: None
            }]))?;
            let v = v.try_as_reflect().ok_or(SerdeError::NotFullReflect(info.type_path()))?;
            self.serialize(v, info)?;
        }
        if trail {
            write!(self.file, "\n)")
        } else {
            write!(self.file, ")")
        }?;
        Ok(())
    }
}