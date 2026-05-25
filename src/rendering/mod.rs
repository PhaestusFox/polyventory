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

    pub use super::InventoryRenderPipeline;

    #[cfg(feature = "sprite_rendering")]
    pub use super::sprite_render::InventorySprite;

    #[cfg(feature = "node_rendering")]
    pub use super::node_render::InventoryNode;
}

pub use style::{InventoryStyle, InventoryStyleAsset};

#[derive(Default)]
pub struct InventoryRenderPlugin {
    pub default_inventory_style: Option<InventoryStyleAsset>,
    pub pipeline: InventoryRenderPipeline,
}

impl Plugin for InventoryRenderPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<InventoryStyle>();
        style::register_default_style(app, self.default_inventory_style.as_ref());
        app.add_observer(render::spawn_inventory_window);

        app.add_systems(Update, render::update_displayed_item_transform);


        #[cfg(feature = "sprite_rendering")]
        app.add_plugins(sprite_render::InventorySpritePlugin {
            auto_require: matches!(self.pipeline, InventoryRenderPipeline::Sprite),
        });
        #[cfg(feature = "node_rendering")]
        app.add_plugins(node_render::InventoryNodePlugin {
            auto_require: matches!(self.pipeline, InventoryRenderPipeline::Node),
        });
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

#[derive(Default)]
/// How an inventory will render by default.
pub enum InventoryRenderPipeline {
    #[default]
    /// The user has opted to declare the pipeline for each `RenderedInventory`
    Custom = 0,
    #[cfg(feature = "node_rendering")]
    /// `RenderedInventory`'s will have `InventoryNode` as a required component
    Node = 1,
    #[cfg(feature = "sprite_rendering")]
    /// `RenderedInventory`'s will have `InventorySprite` as a required component
    Sprite = 2,
}