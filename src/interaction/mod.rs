use crate::{inventory::entry::Entry, prelude::*};
use bevy::prelude::*;

mod interactions;

mod cursor;

mod prelude {
    pub use super::cursor::{DropItem, PickupItem};
    pub use super::interactions::MoveItem;
}

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(cursor::CursorPlugin);
        app.add_observer(interactions::move_item);
    }
}

/// a handle to an inventory slot
/// used to keep track of where and item goes without searching everywhere
pub struct InventoryHandle {
    // the inventory the item came from
    pub(crate) inventory: AssetId<Inventory>,
    // the slot index inside the inventory the item came from
    pub(crate) entry: Shape,
}
