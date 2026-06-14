use bevy::reflect::{ArrayInfo, EnumInfo, GetPath, ListInfo, NamedField, OffsetAccess, ParsedPath, PartialReflect, Reflect, ReflectRef, ReflectSerialize, SetInfo, TypeInfo, UnnamedField};

use crate::serde::error::SerdeError;

impl<T: core::fmt::Write> super::InfoSer<'_, T> {
    pub fn serialise_array(&mut self, v: &dyn Reflect, info: &ArrayInfo) -> Result<(), SerdeError> {
        self.name(info)?;
        self.serialize_seq(v, info.item_info().ok_or(SerdeError::TypeNotRegistered(info.item_ty().path()))?, info.capacity())
    }

    pub fn serialise_list(&mut self, v: &dyn Reflect, info: &ListInfo) -> Result<(), SerdeError> {
        self.name(info)?;
        let ReflectRef::List(l) = v.reflect_ref() else {
            panic!("Non list passed to serialize list");
        };
        self.serialize_seq(v, info.item_info().ok_or(SerdeError::TypeNotRegistered(info.item_ty().path()))?, l.len())
    }

    pub fn serialize_seq<'a>(&mut self, v: &dyn Reflect, item: &TypeInfo, len: usize ) -> Result<(), SerdeError> {
        write!(self.file, "[")?;
        
        for i in 0..len {
            if i != 0 {
                write!(self.file, ",")?;
            }
            // let info = f.type_info().ok_or(SerdeError::TypeNotRegistered(f.type_path()))?;
            let v = v.reflect_path(&ParsedPath(vec![OffsetAccess {
                access: bevy::reflect::Access::ListIndex(i),
                offset: None
            }]))?;
            let v = v.try_as_reflect().ok_or(SerdeError::NotFullReflect(item.type_path()))?;
            write!(self.file, "\n")?;
            self.serialize(v, item)?;
        }
        write!(self.file, "\n]").map_err(SerdeError::WrightError)
    }
}