use std::ops::Deref;

use bevy::reflect::{Reflect, StructInfo, TypeRegistry, TypeRegistryArc};

use crate::serde::error::SerdeError;

pub struct ComponentDeSer;


mod ser;
pub use ser::ComponentSer;

mod de;
pub use de::ComponentDe;

mod naming;
mod tests;
mod error;