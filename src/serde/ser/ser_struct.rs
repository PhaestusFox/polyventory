use bevy::reflect::{StructInfo, TupleStructInfo, TypeRegistry};

use crate::serde::error::SerdeError;

pub struct StructSer<'a, T> {
    file: T,
    reg: &'a TypeRegistry,
    named: bool,
}

impl<'a, T> StructSer<'a, T> {
    pub fn new(file: T, type_registry: &'a TypeRegistry, named: bool) -> Self {
        Self {
            file,
            reg: type_registry,
            named
        }
    }
}

impl<T: core::fmt::Write> StructSer<'_, T> {
    pub fn serialise_struct(&mut self, info: &StructInfo) -> Result<(), SerdeError> {
        if self.named {
            write!(self.file, "{}", info.ty().short_path())?;
            self.named = false;
        }
        if info.field_len() == 0 {
            return Ok(());
        }
        write!(self.file, "(")?;

        write!(self.file, ")").map_err(SerdeError::WrightError)
    }

    pub fn serialize_tuple(&mut self, info: &TupleStructInfo) -> Result<(), SerdeError> {
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
        }
        write!(self.file, ")").map_err(SerdeError::WrightError)
    }
}