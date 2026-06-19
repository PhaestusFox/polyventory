use std::{convert::Infallible, str::FromStr, sync::Arc};

use bevy::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Component, Reflect, Default)]
pub enum CellType {
    Any,
    #[default]
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

impl std::fmt::Display for CellType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CellType::Custom(custom) => write!(f, "{}", custom),
            other => write!(f, "{other:?}"),
        }
    }
}

impl CellType {
    pub fn custom(id: impl Into<Arc<str>>) -> Self {
        Self::Custom(id.into())
    }
}
