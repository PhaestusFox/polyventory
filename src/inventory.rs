use bevy::{ecs::system::SystemParam, math::bounding::Aabb2d, prelude::*};

mod item;
pub mod manager;
mod slot;
mod traits;

pub use item::*;
pub use slot::*;

use crate::inventory::manager::AddFailed;

#[derive(Asset, TypePath)]
pub struct Inventory {
    slots: Vec<Slot>,
}

impl Inventory {
    /// Creates a new inventory with the specified width and height.
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            slots: vec![Slot {
                slot_type: vec![SlotType::Untyped],
                position: IVec2::ZERO,
                size: UVec2::new(width, height),
                entries: vec![],
            }],
        }
    }

    pub fn new_empty() -> Self {
        Self { slots: Vec::new() }
    }

    /// Creates a new inventory with the specified width, height, and slots.
    pub fn new_with_slots(slots: impl Into<Vec<Slot>>) -> Self {
        Self {
            slots: slots.into(),
        }
    }

    /// Add an item to the inventory. in the first available space.
    /// This returns the position the item was placed
    pub fn add_item(
        &mut self,
        item_type: &ItemDescriptor,
        entity: Entity,
    ) -> Option<(usize, Shape, SlotType)> {
        for (i, slot) in self.slots.iter_mut().enumerate() {
            for slot_type in &slot.slot_type {
                // if item cant go in this slot type try the next one
                let Some(shape) = item_type.size(slot_type) else {
                    continue;
                };
                // todo find fist empty space that can fit the item
                if let Some(shape) = slot.fit(&shape) {
                    info!("Added {} to inventory: {:?}", item_type.name(), shape);
                    let entry = Entry {
                        entity,
                        shape: shape.clone(),
                    };
                    let slot_type = slot_type.clone();
                    slot.add_entry(entry);
                    return Some((i, shape, slot_type));
                }
            }
        }
        None
    }

    pub fn add_item_at(
        &mut self,
        item_type: &ItemDescriptor,
        entity: Entity,
        position: IVec2,
        orientation: Orientation,
    ) -> Result<(usize, Shape), Vec<AddFailed>> {
        if self.slots.is_empty() {
            return Err(vec![AddFailed::NoSlotsInInventory]);
        }
        let mut errors = Vec::new();
        for (i, slot) in self.slots.iter_mut().enumerate() {
            for slot_type in &slot.slot_type {
                let Some(mut shape) = item_type.size(slot_type) else {
                    continue;
                };
                shape.position = position - slot.position;
                shape.orientation = orientation;
                match slot.fit_at(&shape) {
                    Ok(()) => {
                        slot.add_entry(Entry {
                            entity,
                            shape: shape.clone(),
                        });
                        trace!(
                            "Added {} to inventory at position {:?} with orientation {:?}",
                            item_type.name(),
                            position,
                            orientation
                        );
                        return Ok((i, shape));
                    }
                    Err(FitFailure::NotInBounds(item, slot)) => {
                        errors.push(AddFailed::NotInBounds {
                            slot_index: i,
                            item_bounds: item,
                            slot_bounds: slot,
                        });
                        break;
                    }
                    Err(FitFailure::OverlapsWith(index, cell)) => {
                        errors.push(AddFailed::OverlapWithExistingItem {
                            slot_index: i,
                            overlap_index: index,
                        });
                        break;
                    }
                }
            }
        }
        if errors.is_empty() {
            errors.push(AddFailed::NoSlotsAcceptThisItem);
        }
        Err(errors)
    }

    /// Add a slot to the inventory.
    pub fn add_slot(&mut self, slot: Slot) {
        self.slots.push(slot);
    }

    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }

    pub fn iter_slots_mut(&mut self) -> impl Iterator<Item = &mut Slot> {
        self.slots.iter_mut()
    }

    pub fn add_unchecked(
        &mut self,
        slot_index: usize,
        shape: Shape,
        entity: Entity,
    ) -> Result<(), AddFailed> {
        let slot = self
            .slots
            .get_mut(slot_index)
            .ok_or(AddFailed::NoSlotsInInventory)?;
        slot.add_entry(Entry { entity, shape });
        Ok(())
    }

    pub fn reserve_item(
        &mut self,
        item_type: &ItemDescriptor,
    ) -> Result<(usize, Shape), AddFailed> {
        if self.slots.is_empty() {
            return Err(AddFailed::NoSlotsInInventory);
        }
        for (i, slot) in self.slots.iter_mut().enumerate() {
            for slot_type in &slot.slot_type {
                // if item cant go in this slot type try the next one
                let Some(shape) = item_type.size(slot_type) else {
                    continue;
                };
                // todo find fist empty space that can fit the item
                if let Some(shape) = slot.fit(&shape) {
                    return Ok((i, shape));
                }
            }
        }
        Err(AddFailed::NoSlotsAcceptThisItem)
    }

    pub fn reserve_item_at(
        &mut self,
        item_type: &ItemDescriptor,
        position: IVec2,
        orientation: Orientation,
    ) -> Result<(usize, Shape, SlotType), AddFailed> {
        if self.slots.is_empty() {
            return Err(AddFailed::NoSlotsInInventory);
        }

        let mut error = AddFailed::NoSlotsInInventory;

        for (i, slot) in self.slots.iter_mut().enumerate() {
            for slot_type in &slot.slot_type {
                let Some(mut shape) = item_type.size(slot_type) else {
                    continue;
                };
                shape.position = position - slot.position;
                shape.orientation = orientation;
                match slot.fit_at(&shape) {
                    Ok(()) => {
                        trace!(
                            "Reserved space for {} in inventory at position {:?} with orientation {:?}",
                            item_type.name(),
                            position,
                            orientation
                        );
                        return Ok((i, shape, slot_type.clone()));
                    }
                    Err(FitFailure::NotInBounds(item, slot)) => {
                        error = AddFailed::NotInBounds {
                            slot_index: i,
                            item_bounds: item,
                            slot_bounds: slot,
                        };
                        break;
                    }
                    Err(FitFailure::OverlapsWith(index, _)) => {
                        error = AddFailed::OverlapWithExistingItem {
                            slot_index: i,
                            overlap_index: index,
                        };
                        break;
                    }
                }
            }
        }
        Err(error)
    }

    #[inline(always)]
    pub fn get_slot(&self, index: usize) -> Option<&Slot> {
        self.slots.get(index)
    }

    pub fn contains(&self, item: Entity) -> bool {
        self.slots.iter().any(|slot| {
            slot.entries
                .iter()
                .any(|entry| entry.entity == item)
        })
    }
}