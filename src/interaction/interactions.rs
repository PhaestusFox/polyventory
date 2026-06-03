use bevy::{ecs::event, prelude::*};
use crate::{inventory::entry::Entry, prelude::*};

#[derive(Debug, Event)]
pub enum MoveItem {
    PlaceItem {
        item: Entity,
        inventory: AssetId<Inventory>,
        cell_type: CellType,
        shape: Shape,
    },
    FitItem {
        item: Entity,
        inventory: AssetId<Inventory>,
    },
    InsertItem {
        item: Entity,
        inventory: AssetId<Inventory>,
        cell_type: CellType,
        shape: Shape,
    }
}

impl MoveItem {
    pub fn item(&self) -> Entity {
        match self {
            MoveItem::PlaceItem { item, .. } => *item,
            MoveItem::FitItem { item, .. } => *item,
            MoveItem::InsertItem { item, .. } => *item,
        }
    }

    pub fn destination(&self) -> AssetId<Inventory> {
        match self {
            MoveItem::PlaceItem { inventory, .. } => *inventory,
            MoveItem::FitItem { inventory, .. } => *inventory,
            MoveItem::InsertItem { inventory, .. } => *inventory
        }
    }
}

pub fn move_item(
    event: On<MoveItem>,
    mut manager: InventoryManager,
) {
    let Some(origin) = manager.find_item(event.item()) else {
        trace!("Failed to find item {:?} in any inventory", event.item());
        return;
    };
    let Some(mut dest) = manager.open_inventory(event.destination()) else {
        trace!("Failed to open destination inventory for item {:?} move", event.item());
        return;
    };
    let target = match event.event() {
        MoveItem::FitItem {..} => {
            dest.find_fit(event.item()).ok()
        },
        MoveItem::PlaceItem { cell_type, shape, .. } => {
            if dest.can_fit(cell_type, shape) {
                Some((cell_type.clone(), shape.clone()))
            } else {
                None
            }
        },
        MoveItem::InsertItem { item, cell_type, shape, .. } => {
            dest.insert_item(*item, shape.clone(), cell_type.clone());
            return;
        }
    };
    error!("Moving Item");
    let Some((cell, shape)) = target else {
        trace!("Failed to fit item {:?} inventory {:?}", event.item(), event.destination());
        return;
    };
    if let Err(e) = dest.add_item(event.item(), shape) {
        warn!("Failed to move item {}", e);
    }
}