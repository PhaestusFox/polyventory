use std::io::{Read, Seek};

use bevy::reflect::{ArrayInfo, DynamicArray, DynamicEnum, DynamicList, DynamicStruct, DynamicTuple, DynamicVariant, EnumInfo, List, NamedField, PartialReflect, Reflect, ReflectDeserialize, ReflectFromReflect, StructInfo, TupleInfo, TupleStructInfo, TypeInfo, UnnamedField};

use crate::serde::error::DeError;

impl<T: Read + Seek> super::InfoDe<'_, T> {
    pub fn deserialize_tuple(&self, data: &str, info: &TupleInfo) -> Result<Box<dyn PartialReflect>, DeError> {
        let fields = info.iter().collect::<Vec<_>>();
        let mut d = self.deserialize_tuples(data, &fields)?;
        match self.from_reflect(info.ty(), d.as_partial_reflect()) {
            Ok(a) => Ok(a),
            Err(_) => {
                d.set_represented_type(self.get_info(info.type_id()));
                Ok(Box::new(d))
            },
        }
    }

    pub fn deserialize_tuples(&self, data: &str, fields: &[&UnnamedField]) -> Result<DynamicTuple, DeError> {
        let data = data.trim();
        assert!(data.starts_with('(') && data.ends_with(')'), "{}", data);
        let mut data = &data[1..data.len()-1];
        let mut dyn_struct = DynamicTuple::default();
        for f in fields.iter() {
            let (block, rest) = super::extract_ron_blob(data)?;
            let v = self.deserialize_str(block, f.type_info().unwrap())?;
            dyn_struct.insert_boxed(v);
            data = rest;
        }
        Ok(dyn_struct)
    }
}