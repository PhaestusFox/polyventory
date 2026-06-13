use bevy::reflect::{EnumInfo, PartialReflect, Reflect, ReflectSerialize};

use crate::serde::error::SerdeError;

impl<T: core::fmt::Write> super::InfoSer<'_, T> {
    pub fn serialise_enum(&mut self, v: &dyn Reflect, info: &EnumInfo) -> Result<(), SerdeError> {
        if self.named {
            write!(self.file, "{}", info.ty().short_path())?;
            self.named = false;
        };
        
        Ok(())
    }
}