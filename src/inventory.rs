use bevy::{ecs::{entity::EntityHashMap, system::SystemParam}, math::bounding::Aabb2d, platform::collections::{HashMap, HashSet}, prelude::*};

mod item;
pub mod manager;
mod traits;

mod cell_type;
pub use cell_type::CellType;

mod shape;
pub use shape::*;
pub mod entry;
pub use item::*;
use strum::IntoEnumIterator;

// mod slot;
// pub use slot::*;

use crate::{inventory::manager::AddFailed};

mod inventory_descriptor;
pub use inventory_descriptor::{InventoryDescriptor, InventoryDescriptorLoader};

#[derive(Asset, Reflect, Default)]
pub struct Inventory {
    name: String,
    slots: HashMap<CellType, shape::Shape>,
    items: EntityHashMap<entry::Entry>,
}

impl Inventory {
    /// Creates a new inventory with the specified width and height.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            slots: HashMap::default(),
            items: EntityHashMap::default(),
        }
    }

    /// Creates a new inventory with the specified width, height, and slots.
    pub fn new_with_slots(slots: impl Iterator<Item = (CellType, shape::Shape)>) -> Self {
        Self {
            name: String::new(),
            slots: slots.collect(),
            items: EntityHashMap::default(),
        }
    }

    /// Add an item to the inventory. in the first available space.
    /// This returns the position the item was placed
    fn add_item(
        &mut self,
        item_type: &ItemDescriptor,
        entity: Entity,
        sub_inv: Option<AssetId<Inventory>>,
    ) -> Option<(CellType, shape::Shape)> {
        for (slot_type, layout) in item_type.sizes() {
            // todo find fist empty space that can fit the item
            let shape = Shape {
                layout: layout.clone(),
                offset: IVec2::ZERO,
                orientation: Orientation::Deg0,
            };
            if self.can_fit(slot_type, &shape) {
                info!("Added {} to inventory: {:?}", item_type.name(), shape);
                let entry = entry::Entry {
                    shape: shape.clone(),
                    sub_inventory: sub_inv,
                };
                let slot_type = slot_type.clone();
                self.items.insert(entity, entry);
                return Some((slot_type, shape.clone()));
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
        todo!()
    }

    /// Add a slot to the inventory.
    pub fn add_slot(&mut self, cell_type: CellType, shape: shape::Shape) {
        self.slots.insert(cell_type, shape);
    }

    /// Returns true if the inventory contains the item, directly
    pub fn contains(&self, item: Entity) -> bool {
        self.items.contains_key(&item)
    }

    /// remove all instances of an item from an inventory
    pub fn remove(&mut self, item: Entity) -> Option<entry::Entry> {
        self.items.remove(&item)
    }

    pub fn fit(&self, item_type: &ItemDescriptor) -> Option<(CellType, Shape)> {
        for (slot_type, item_layout) in item_type.sizes() {
            let Some(slot_layout) = self.slots.get(slot_type) else {
                continue;
            };
            for orientation in Orientation::iter() {
                let mut shape = Shape {
                    layout: item_layout.clone(),
                    offset: IVec2::ZERO,
                    orientation,
                };
                for cell in slot_layout.iter_cells() {
                    shape.offset = cell;
                    if self.can_fit(slot_type, &shape) {
                        println!("Found fit for {} in inventory: {:?}", item_type.name(), shape);
                        return Some((slot_type.clone(), shape));
                    }
                }
            }
        }
        None
    }

    pub fn can_fit(&self, cell_type: &CellType, item_shape: &shape::Shape) -> bool {
        // incompatible slot type
        let Some(slot_shape) = self.slots.get(cell_type) else {
            return false;
        };
        if !slot_shape.can_fit(item_shape) {
            return false;
        }
        let item_bounds = item_shape.bounds();
        for item in self.items.values() {
            let used_bounds = item.shape.bounds();
            // if the item bounds don't intersect with the new item bounds, we can skip detailed checking
            if item_bounds <= used_bounds {
                return false;
            }
            // TODO invert if to skip items we are not in bounds of and then check if we overlap with the item
            // for cell in item_bounds.min(used_bounds).iter() {
            //     if item.shape.contains(cell) && item_shape.contains(cell) {
            //         return false;
            //     }
            // }
        }
        true
    }

    pub fn iter_sub_inventories(&self) -> impl Iterator<Item = AssetId<Inventory>> + '_ {
        self.items.values().filter_map(|entry| entry.sub_inventory)
    }

    pub fn insert_item(&mut self, item: Entity, entry: entry::Entry) {
        self.items.insert(item, entry);
    }

    pub fn slots(&self) -> impl Iterator<Item = (&CellType, &shape::Shape)> {
        self.slots.iter()
    }

    pub fn items(&self) -> impl Iterator<Item = (&Entity, &entry::Entry)> {
        self.items.iter()
    }

    pub fn get_shape(&self, item: Entity) -> Option<&shape::Shape> {
        self.items.get(&item).map(|entry| &entry.shape)
    }

    pub fn find(&self, item: Entity, assets: &Assets<Inventory>, checked: &mut HashSet<AssetId<Inventory>>) -> Option<FoundItem> {
        if self.contains(item) {
            return Some(FoundItem::InSelf);
        }
        for entry in self.iter_sub_inventories() {
            if checked.contains(&entry) {
                error!("Inventory {:?} has a circular reference. Already checked this inventory while looking for item {:?}. Skipping.", entry, item);
                continue;
            }
            checked.insert(entry);
            let Some(inventory) = assets.get(entry) else {
                warn!("Failed to get inventory {:?} while trying to find item {:?}. Skipping.", entry, item);
                continue;
            };
            match inventory.find(item, assets, checked) {
                Some(FoundItem::InSelf) => return Some(FoundItem::InSubInventory(entry)),
                Some(FoundItem::InSubInventory(sub)) => return Some(FoundItem::InSubInventory(sub)),
                None => continue,
            }
        }
        None
    }
}

pub enum FoundItem {
    InSelf,
    InSubInventory(AssetId<Inventory>),
}

pub(crate) mod inventory_relationship;