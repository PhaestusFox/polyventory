use std::{convert::Infallible, str::FromStr, sync::Arc};

use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Component, Reflect)]
pub enum CellType {
    Untyped,
    WaterTight,
    Small,
    Custom(Arc<str>),
}

impl FromStr for CellType {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Untyped" => Ok(CellType::Untyped),
            "WaterTight" => Ok(CellType::WaterTight),
            "Small" => Ok(CellType::Small),
            custom => Ok(CellType::Custom(Arc::from(custom))),
        }
    }
}