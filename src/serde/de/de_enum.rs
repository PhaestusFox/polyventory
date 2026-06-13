use std::io::{Read, Seek};

use bevy::reflect::{ArrayInfo, DynamicArray, DynamicEnum, DynamicList, DynamicStruct, DynamicTuple, DynamicVariant, EnumInfo, List, NamedField, PartialReflect, Reflect, ReflectDeserialize, ReflectFromReflect, TypeInfo, UnnamedField};
use bevy_inspector_egui::egui::Key::P;

use crate::serde::error::DeError;

impl<T: Read + Seek> super::InfoDe<'_, T> {
    pub fn deserialize_enum(&self, data: &str, info: &EnumInfo) -> Result<Box<dyn PartialReflect>, DeError> {
        let (name, data) = split_enum(data);
        match info.variant(name).ok_or(DeError::UnknownVariant(info.ty().short_path(), String::from(name)))? {
            bevy::reflect::VariantInfo::Struct(struct_variant_info) => {
                let fields = struct_variant_info.iter().collect::<Vec<_>>();
                let s = self.deserialize_structs(data, &fields)?;
                let r = DynamicEnum::new(name, DynamicVariant::Struct(s));
                self.from_reflect(info.ty(), r.as_partial_reflect())
            },
            bevy::reflect::VariantInfo::Tuple(tuple_variant_info) => {
                let fields = tuple_variant_info.iter().collect::<Vec<_>>();
                let s = self.deserialize_tuples(data, &fields)?;
                let r = DynamicEnum::new(name, DynamicVariant::Tuple(s));
                self.from_reflect(info.ty(), r.as_partial_reflect())
            },
            bevy::reflect::VariantInfo::Unit(unit_variant_info) => {
                let r = DynamicEnum::new(name, DynamicVariant::Unit);
                self.from_reflect(info.ty(), r.as_partial_reflect())
            },
        }.map(|v| v.into_partial_reflect())
    }
}

fn split_enum(block: &str) -> (&str, &str) {
    let block = block.trim_matches(|c: char| c.is_whitespace() || c == ':');
    let Some(f) = block.find(|c: char| c == '(' || c == '{') else {
        return (block, "")
    };
    block.split_at(f)
}