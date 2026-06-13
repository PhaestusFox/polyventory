use std::io::{Read, Seek};

use bevy::reflect::{ArrayInfo, DynamicArray, DynamicEnum, DynamicList, DynamicStruct, DynamicTuple, DynamicTupleStruct, DynamicVariant, EnumInfo, List, NamedField, PartialReflect, Reflect, ReflectDeserialize, ReflectFromReflect, StructInfo, TupleStructInfo, TypeInfo, UnnamedField};

use crate::serde::error::DeError;

impl<T: Read + Seek> super::InfoDe<'_, T> {

    pub fn deserialize_struct(&self, data: &str, info: &StructInfo) -> Result<Box<dyn PartialReflect>, DeError> {
        if info.field_len() == 0 {
            let d = DynamicStruct::default();
            return self.from_reflect(info.ty(), d.as_partial_reflect());
        }
        let fields = info.iter().collect::<Vec<_>>();
        let s = self.deserialize_structs(data, &fields)?;
        self.from_reflect(info.ty(), s.as_partial_reflect())
    }
    pub fn deserialize_struct_tuple(&self, data: &str, info: &TupleStructInfo) -> Result<Box<dyn PartialReflect>, DeError> {
        let fields = info.iter().collect::<Vec<_>>();
        let s = self.deserialize_tuples(data, &fields)?;
        let d = DynamicTupleStruct::from_iter(s);
        self.from_reflect(info.ty(), d.as_partial_reflect())
    }

    pub fn deserialize_structs(&self, data: &str, fields: &[&NamedField]) -> Result<DynamicStruct, DeError> {
        let data = data.trim();
        assert!(data.starts_with('{') && data.ends_with('}'), "{}", data);
        let mut data = &data[1..data.len()-1];
        let mut dyn_struct = DynamicStruct::default();
        loop {
            let (block, rest) = super::extract_ron_blob(data)?;
            let (field, block) = split_filed(block).unwrap();
            let info = fields.iter().find(|v| v.name() == field).unwrap();
            let v = self.deserialize_str(block, info.type_info().unwrap())?;
            dyn_struct.insert_boxed(field, v);
            if rest.is_empty() {
                return Ok(dyn_struct);
            }
            data = rest;
        }
    }
}

fn split_filed(block: &str) -> Option<(&str, &str)> {
    block.split_once(':')
}