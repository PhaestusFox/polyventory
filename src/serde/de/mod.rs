use std::{any::TypeId, io::{BufRead, Cursor, Read, Seek}};

use bevy::reflect::{PartialReflect, Reflect, Type, TypeInfo, TypeRegistration, TypeRegistry, TypeRegistryArc};

use crate::serde::error::DeError;

mod de_opaque;
mod de_array;
mod de_enum;
mod de_struct;
mod de_tuple;

pub struct ComponentDe {
    pub type_repo: TypeRegistryArc,
}

impl<'de> ComponentDe {
    pub fn deserialize_str(&self, data: &str) -> Result<Box<dyn PartialReflect>, DeError> {
        let r = self.type_repo.read();
        let mut de = InfoDe {
            file: Cursor::new(data),
            registry: &r,
        };
        de.deserialize()
    }
}

pub struct InfoDe<'a, T> {
    pub(super) file: T,
    pub(super) registry: &'a TypeRegistry,
}

impl<T> InfoDe<'_, T> {
    pub fn from_reflect(&self, id: &Type, from: &dyn PartialReflect) -> Result<Box<dyn PartialReflect>, DeError> {
        let fr = self.registry.get_type_data::<bevy::reflect::ReflectFromReflect>(id.id()).ok_or(DeError::NoFromReflect(id.short_path()))?;
        fr.from_reflect(from).map(|f| f.into_partial_reflect()).ok_or(DeError::FromReflectFailed(id.short_path()))
    }

    pub fn get_info(&self, id: TypeId) -> Option<&'static TypeInfo> {
        self.registry.get_type_info(id)
    }
}

impl<T: Read + Seek> InfoDe<'_, T> {
    pub fn deserialize(&mut self) -> Result<Box<dyn PartialReflect>, DeError> {
        // read the start of the file to work out what struct we are trying to read
        let info = self.get_type()?;
        let data = self.extract_ron_blob()?;
        if let Some(c) = self.deserialize_custom(&data, info)? {
            Ok(c)
        } else {
            dbg!(&data);
            dbg!(info);
            self.deserialize_str(&data, info)
        }
    }

    pub fn get_type(&mut self) -> Result<&'static TypeInfo, DeError> {
        let mut stack = Vec::new();
        let mut first = true;
        let mut name = String::new();
        let mut next = [0; 256];
        let mut read = 0;
        'scan: while let Ok(r) = self.file.read(&mut next) {
            if r == 0 {
                return Err(DeError::NoTypeFound{
                    start: 0,
                    end: read
                });
            }
            read += r;
            for (i, &b) in next[..r].into_iter().enumerate() {
                let char = b as char;
                if first && (char.is_whitespace() || char == ':') {
                    continue;
                }
                name.push(char);

                match char {
                    // if hit opening brace && stack is empty && not the very first char
                    // we have found the end of the name and just opened the data
                    '[' | '(' | '{' | ':' if stack.is_empty() && !first => {
                        self.file.seek(std::io::SeekFrom::Current(i as i64 - r as i64))?;
                        name.pop();
                        break 'scan;
                    }
                    // if opening brace
                    // push on nest stack
                    '[' | '(' | '<' | '{' => {
                        stack.push(recp(char));
                    }
                    // if close brace && match last open
                    // pop off nest
                    ']' | ')' | '>' | '}' if stack.last() == Some(&char) => {
                        stack.pop();
                    }
                    _ => {}
                }
                first = false;
            }
        }
        if self.registry.is_ambiguous(&name) {
            return Err(DeError::AmbiguousType(name));
        }
        self.registry.get_with_short_type_path(&name).map(|v| v.type_info()).ok_or(DeError::TypeNotRegistered(name))
    } 

    pub fn deserialize_str(&self, data: &str, info: &TypeInfo) -> Result<Box<dyn PartialReflect>, DeError> {
        match info {
            bevy::reflect::TypeInfo::Struct(struct_info) => self.deserialize_struct(data, struct_info),
            bevy::reflect::TypeInfo::TupleStruct(tuple_struct_info) => self.deserialize_struct_tuple(data, tuple_struct_info),
            bevy::reflect::TypeInfo::Tuple(tuple_info) => self.deserialize_tuple(data, tuple_info),
            bevy::reflect::TypeInfo::List(list_info) => self.deserialize_list(data, list_info),
            bevy::reflect::TypeInfo::Array(array_info) => self.deserialize_array(data, array_info),
            bevy::reflect::TypeInfo::Map(map_info) => todo!(),
            bevy::reflect::TypeInfo::Set(set_info) => todo!(),
            bevy::reflect::TypeInfo::Enum(enum_info) => self.deserialize_enum(data, enum_info),
            bevy::reflect::TypeInfo::Opaque(opaque_info) => self.deserialize_opaque(data, opaque_info),
        }
    }

    pub fn deserialize_custom(&self, data: &str, info: &TypeInfo) -> Result<Option<Box<dyn PartialReflect>>, DeError> {
        if info.type_id() == TypeId::of::<()>() {
            Ok(Some(Box::new(())))
        } else {
            Ok(None)
        }
    }
}

const fn recp(char: char) -> char {
    match char {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '<' => '>',
        _ => char,
    }
}

pub fn extract_ron_blob(data: &str) -> Result<(&str, &str), DeError> {
        let mut stack = Vec::new();
        let data = data.trim_start_matches(|char: char| char.is_whitespace() || char == ',');
        for (i, c) in data.char_indices() {
            match c {
                // if opening brace
                // push on nest stack
                '[' | '(' | '{' => {
                    stack.push(recp(c));
                }
                // if close brace && match last open
                // pop off nest
                ']' | ')' | '}' if stack.last() == Some(&c) => {
                    stack.pop();
                }
                ']' | ')' | '}' | ',' if stack.is_empty() => {
                    return Ok(data.split_at(i));
                }
                _ => {}
            }
        }
        Ok((data, ""))
    }