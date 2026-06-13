use std::ops::DerefMut;

use bevy::reflect::{GetPath, OffsetAccess, ParsedPath, Reflect, StructInfo, TupleStructInfo, TypeRegistry};

use crate::serde::error::SerdeError;

impl<T: core::fmt::Write> super::InfoSer<'_, T> {
    pub fn serialise_struct(&mut self, v: &dyn Reflect, info: &StructInfo) -> Result<(), SerdeError> {
        if self.named {
            write!(self.file, "{}", info.ty().short_path())?;
            self.named = false;
        }
        if info.field_len() == 0 {
            return Ok(());
        }
        write!(self.file, "{{")?;
        if info.field_len() > 0 {
            write!(self.file, "\n")?;
        }

        for (i, f) in info.iter().enumerate() {
            if i != 0 {
                write!(self.file, ",\n")?;
            };
            let info = f.type_info().ok_or(SerdeError::TypeNotRegistered(f.type_path()))?;
            let v = v.reflect_path(&ParsedPath(vec![OffsetAccess {
                access: bevy::reflect::Access::FieldIndex(i),
                offset: None
            }]))?;
            let v = v.try_as_reflect().ok_or(SerdeError::NotFullReflect(info.type_path()))?;
            write!(self.file, "{}:", f.name());
            self.serialize(v, info)?;
        }

        write!(self.file, "}}").map_err(SerdeError::WrightError)
    }

    pub fn serialize_tuple(&mut self, v: &dyn Reflect, info: &TupleStructInfo) -> Result<(), SerdeError> {
        if self.named {
            write!(self.file, "{}", info.ty().short_path())?;
            self.named = false;
        };
        write!(self.file, "(")?;
        if info.field_len() > 1 {
            write!(self.file, "\n")?;
        }
        for f in info.iter() {
            if f.index() != 0 {
                write!(self.file, ",\n")?;
            }
            let info = f.type_info().ok_or(SerdeError::TypeNotRegistered(f.type_path()))?;
            let v = v.reflect_path(&ParsedPath(vec![OffsetAccess {
                access: bevy::reflect::Access::TupleIndex(f.index()),
                offset: None
            }]))?;
            let v = v.try_as_reflect().ok_or(SerdeError::NotFullReflect(info.type_path()))?;
            self.serialize(v, info)?;
        }
        if info.field_len() > 1 {
            write!(self.file, "\n")?;
        }
        write!(self.file, ")").map_err(SerdeError::WrightError)
    }
}