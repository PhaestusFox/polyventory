use bevy::prelude::*;

use crate::inventory::Inventory;

use super::shape::Shape;

#[derive(Reflect)]
/// Represents an item in an inventory, including its shape and if it has a sub-inventory
pub struct Entry {
    /// the entity of the item, uses to store additional data
    pub entity: Entity,
    /// the shape of the item, in the inventory it occupies
    pub shape: Shape,
    /// if the item has a sub-inventory, this is the asset id of it
    pub sub_inventory: Option<AssetId<Inventory>>,
}