use std::io::{Read, Seek};

use bevy::reflect::{ArrayInfo, DynamicArray, DynamicList, List, ListInfo, PartialReflect, Reflect, ReflectDeserialize, ReflectFromReflect, TypeInfo};

use crate::serde::error::DeError;

impl<T: Read + Seek> super::InfoDe<'_, T> {
    pub fn deserialize_array(&self, data: &str, info: &ArrayInfo) -> Result<Box<dyn PartialReflect>, DeError> {
        let found = self.deserialize_seq(data, info.item_info().ok_or(DeError::TypeNotRegistered(info.item_ty().path().to_string()))?)?;
        if found.len() != info.capacity() {
            return Err(DeError::NotEnoughItems(found.capacity(), info.capacity()));
        }
        let f = found.into_iter().map(|v| v.into_partial_reflect()).collect::<Vec<_>>();
        let mut array = DynamicArray::new(f.into_boxed_slice());
        dbg!(&array);
        match self.from_reflect(info.ty(), array.as_partial_reflect()) {
            Ok(a) => Ok(a),
            Err(_) => {
                array.set_represented_type(self.get_info(info.type_id()));
                Ok(Box::new(array))
            },
        }
    }

    pub fn deserialize_list(&self, data: &str, info: &ListInfo) -> Result<Box<dyn PartialReflect>, DeError> {
        let found = self.deserialize_seq(data, info.item_info().ok_or(DeError::TypeNotRegistered(info.item_ty().path().to_string()))?)?;
        let mut list = DynamicList::from_iter(found);
        match self.from_reflect(info.ty(), list.as_partial_reflect()) {
            Ok(a) => Ok(a),
            Err(_) => {
                list.set_represented_type(self.get_info(info.type_id()));
                Ok(Box::new(list))
            },
        }
    }

    pub fn deserialize_seq(&self, data: &str, item: &TypeInfo) -> Result<Vec<Box<dyn PartialReflect>>, DeError> {
        let data = data.trim();
        assert!(data.starts_with('[') && data.ends_with(']'));
        let mut data = &data[1..data.len()-1];
        let mut found = Vec::new();
        loop {
            let (block, rest) = super::extract_ron_blob(data)?;
            let v = self.deserialize_str(block, item)?;
            found.push(v);
            if rest.is_empty() {
                return Ok(found);
            }
            data = rest;
        }
    }


}