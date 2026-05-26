use std::str::FromStr;

use bevy::asset::{AssetLoader, AsyncReadExt};
use bevy::platform::collections::HashMap;

use crate::inventory::shape::Layout;

use super::shape::Shape;
use super::*;

#[derive(Asset, Reflect)]
pub struct InventoryDescriptor {
    slots: HashMap<SlotType, Shape>,
}

impl InventoryDescriptor {
    pub fn create_inventory(&self) -> Inventory {
        let mut inventory = Inventory::new_empty();
        for (slot_type, shape) in &self.slots {
            println!("adding inv slot: {:?}:{:?}", slot_type, shape);
            inventory.add_slot(Slot {
                slot_type: vec![slot_type.clone()],
                position: shape.offset,
                size: match shape.layout {
                    Layout::Rect { size } => size,
                },
                entries: vec![],
            });
        }
        inventory
    }
}

impl FromStr for InventoryDescriptor {
    type Err = InventoryDescriptorParseError;

    fn from_str(data: &str) -> Result<Self, Self::Err> {
        let mut slots = HashMap::new();
        let mut slot = None;
        for line in data.lines() {
            let line = line.trim();
            if line.starts_with('{') {
                if let Some((slot_type, shape)) = slot.take() {
                    slots.insert(slot_type, shape);
                }
                slot = Some((SlotType::Untyped, Shape::default()));
            }
            let line = line.trim_start_matches(|c: char| c == '{' || c.is_whitespace());
            let Some((id, val)) = line.split_once(':') else {
                continue;
            };
            match id.trim() {
                "slot" => {
                    let t = SlotType::from_str(val.trim()).expect("any string is a valid slot");
                    if let Some((slot_type, _)) = slot.as_mut() {
                        *slot_type = t;
                    } else {
                        slot = Some((t, Shape::default()));
                    }
                },
                "x" => {
                    let x = val.trim().parse().map_err(|e| InventoryDescriptorParseError::FieldParseError("x", e))?;
                    if let Some((_, shape)) = slot.as_mut() {
                        shape.offset.x = x;
                    } else {
                        slot = Some((SlotType::Untyped, Shape {
                            offset: IVec2::new(x, 0),
                            ..default()
                        }));
                    }
                }
                "y" => {
                    let y = val.trim().parse().map_err(|e| InventoryDescriptorParseError::FieldParseError("y", e))?;
                    if let Some((_, shape)) = slot.as_mut() {
                        shape.offset.y = y;
                    } else {
                        slot = Some((SlotType::Untyped, Shape {
                            offset: IVec2::new(0, y),
                            ..default()
                        }));
                    }
                },
                "w" => {
                    let w = val.trim().parse().map_err(|e| InventoryDescriptorParseError::FieldParseError("w", e))?;
                    if let Some((_, shape)) = slot.as_mut() {
                        match shape.layout {
                            Layout::Rect { ref mut size } => {
                                size.x = w;
                            }
                        }
                    } else {
                        slot = Some((SlotType::Untyped, Shape {
                            layout: Layout::Rect { size: UVec2::new(w, 0) },
                            ..default()
                        }));
                    }
                }
                "h" => {
                    let h = val.trim().parse().map_err(|e| InventoryDescriptorParseError::FieldParseError("h", e))?;
                    if let Some((_, shape)) = slot.as_mut() {
                        match shape.layout {
                            Layout::Rect { ref mut size } => {
                                size.y = h;
                            }
                        }
                    } else {
                        slot = Some((SlotType::Untyped, Shape {
                            layout: Layout::Rect { size: UVec2::new(0, h) },
                            ..default()
                        }));
                    }
                }
                _ => {}
            }
        }
        if let Some((slot_type, shape)) = slot.take() {
            slots.insert(slot_type, shape);
        }
        Ok(Self { slots })
    }
}

#[derive(Default, TypePath)]
pub struct InventoryDescriptorLoader;

impl AssetLoader for InventoryDescriptorLoader {
    type Asset = InventoryDescriptor;
    type Settings = ();
    type Error = InventoryDescriptorLoadError;
    
    fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        _load_context: &mut bevy::asset::LoadContext,
    ) -> impl bevy::tasks::ConditionalSendFuture<Output = std::prelude::v1::Result<Self::Asset, Self::Error>> {
        async {
            let mut data = String::new();
            reader.read_to_string(&mut data).await?;
            Ok(InventoryDescriptor::from_str(&data)?)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InventoryDescriptorLoadError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Parse error: {0}")]
    ParseError(#[from] InventoryDescriptorParseError),
}

#[derive(Debug, thiserror::Error)]
pub enum InventoryDescriptorParseError {
    #[error("Failed to parse {0} value: {1}")]
    FieldParseError(&'static str, std::num::ParseIntError),
}
