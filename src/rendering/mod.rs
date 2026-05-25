use bevy::prelude::*;
use crate::prelude::*;

mod render;
#[cfg(feature = "node_rendering")]
pub mod node_render;

#[cfg(feature = "sprite_rendering")]
pub mod sprite_render;

pub mod interaction;
mod style;

pub mod render_prelude {
    pub use super::InventoryRenderPlugin;
    pub use super::render::{DisplayedItem, SpawnInventory};
    pub use super::style::*;
    pub use super::{RenderedInventory, RenderedSlot};

    #[cfg(feature = "sprite_rendering")]
    pub use super::sprite_render::InventorySprite;

    #[cfg(feature = "node_rendering")]
    pub use super::node_render::InventoryNode;
}

pub use style::{InventoryStyle, InventoryStyleAsset};

#[derive(Default)]
pub struct InventoryRenderPlugin {
    pub default_inventory_style: Option<InventoryStyleAsset>,
}

impl Plugin for InventoryRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<InventoryStyle>();
        style::register_default_style(app, self.default_inventory_style.as_ref());
        app.add_observer(render::spawn_inventory_window);

        app.add_systems(Update, render::update_displayed_item_transform);
    }
}

#[derive(Component)]
#[relationship_target(relationship = RenderedSlot)]
pub struct RenderedInventory {
    inventory: Handle<Inventory>,
    #[relationship]
    slots: Vec<Entity>,
}

impl RenderedInventory {
    pub fn new(inventory: Handle<Inventory>) -> Self {
        Self { inventory, slots: Vec::new() }
    }

    pub fn get_slot(&self, index: usize) -> Option<Entity> {
        self.slots.get(index).cloned()
    }
}

impl Into<AssetId<Inventory>> for &RenderedInventory {
    fn into(self) -> AssetId<Inventory> {
        self.inventory.id()
    }
}

#[derive(Component)]
#[relationship(relationship_target = RenderedInventory)]
#[component(immutable)]
pub struct RenderedSlot {
    #[relationship]
    pub(crate) inventory: Entity,
    pub(crate) index: usize,
}

impl RenderedSlot {
    pub fn index(&self) -> usize {
        self.index
    }
}