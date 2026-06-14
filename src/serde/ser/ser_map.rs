use bevy::reflect::{ArrayInfo, EnumInfo, GetPath, ListInfo, MapInfo, NamedField, OffsetAccess, ParsedPath, PartialReflect, Reflect, ReflectRef, ReflectSerialize, SetInfo, TypeInfo, UnnamedField};

use crate::serde::error::SerdeError;

impl<T: core::fmt::Write> super::InfoSer<'_, T> {
    pub fn serialize_map<'a>(&mut self, v: &dyn Reflect, info: &MapInfo) -> Result<(), SerdeError> {
        self.name(info);
        let ReflectRef::Map(l) = v.reflect_ref() else {
            panic!("Non map passed to serialize map");
        };
        let key = info.key_info().ok_or(SerdeError::TypeNotRegistered(info.key_ty().path()))?;
        let val = info.value_info().ok_or(SerdeError::TypeNotRegistered(info.value_ty().path()))?;
        write!(self.file, "{{")?;
        
        for (i, (k, v)) in l.iter().enumerate() {
            if i != 0 {
                write!(self.file, ",")?;
            }

            let k = k.try_as_reflect().ok_or(SerdeError::NotFullReflect(info.key_ty().path()))?;
            write!(self.file, "\n")?;
            self.serialize(k, key)?;
            write!(self.file, ": ")?;
            let v = v.try_as_reflect().ok_or(SerdeError::NotFullReflect(info.value_ty().path()))?;
            self.serialize(v, val)?;
        }
        write!(self.file, "\n}}").map_err(SerdeError::WrightError)
    }

    pub fn serialize_set(&mut self, v: &dyn Reflect, info: &SetInfo) -> Result<(), SerdeError> {
        self.name(info);
        let ReflectRef::Set(l) = v.reflect_ref() else {
            panic!("Non set passed to serialize set");
        };
        let info = self.registry.get_type_info(info.value_ty().id()).ok_or(SerdeError::TypeNotRegistered(info.value_ty().path()))?;
        write!(self.file, "{{")?;
        
        for (i, v) in l.iter().enumerate() {
            if i != 0 {
                write!(self.file, ",")?;
            }

            let v = v.try_as_reflect().ok_or(SerdeError::NotFullReflect(info.type_path()))?;
            write!(self.file, "\n")?;
            self.serialize(v, info)?;
        }
        write!(self.file, "\n}}").map_err(SerdeError::WrightError)
    }
}
