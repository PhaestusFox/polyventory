use std::io::{Read, Seek};

use bevy::reflect::{OpaqueInfo, PartialReflect, Reflect, ReflectDeserialize};

use crate::serde::error::DeError;

impl<T: Read + Seek> super::InfoDe<'_, T> {
    pub fn deserialize_opaque(&self, data: &str, info: &OpaqueInfo) -> Result<Box<dyn PartialReflect>, DeError> {
        // if we get here and print the name of an opaque type we need to add a : to split the value
        let data = data.trim_start_matches(|char: char| char.is_whitespace() || char == ':');
        let reg = self.registry.get_type_data::<ReflectDeserialize>(info.type_id()).ok_or(DeError::OpaqueNotSerde(info.type_path()))?;
        let mut ser = ron::Deserializer::from_str(data).expect("Can turn string into ron::Deserializer");
        let v = reg.deserialize(&mut ser)?;
        println!("turn {:?} into {}: {:?}", data, info.ty().short_path(), v);
        Ok(v)
    }

    pub fn extract_ron_blob(&mut self) -> Result<String, DeError> {
        let mut data = String::new();
        let mut buff = [0; 256];
        let mut stack = Vec::new();
        loop {
            let r = self.file.read(&mut buff)?;
            if r == 0 {
                return Ok(data);
            }
            for (i, b) in buff[..r].into_iter().enumerate() {
                let char = *b as char;
                data.push(char);
                match char {
                    // if opening brace
                    // push on nest stack
                    '[' | '(' | '{' => {
                        stack.push(super::recp(char));
                    }
                    // if close brace && match last open
                    // pop off nest
                    ']' | ')' | '}' if stack.last() == Some(&char) => {
                        stack.pop();
                    }
                    ']' | ')' | ',' | '}' if stack.is_empty() => {
                        self.file.seek(std::io::SeekFrom::Current(i as i64 - r as i64))?;
                        return Ok(data);
                    }
                    _ => {}
                }
            }
        }
    }
}