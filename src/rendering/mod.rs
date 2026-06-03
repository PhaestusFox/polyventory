use std::fmt::Debug;

use bevy::{ecs::{lifecycle::HookContext, world::DeferredWorld}, prelude::*};
use crate::prelude::*;

mod render;
#[cfg(feature = "node_rendering")]
pub mod node_render;

#[cfg(feature = "sprite_rendering")]
pub mod sprite_render;

#[cfg(feature = "tooltips")]
pub mod tooltip;
mod style;

pub mod render_prelude {
    pub use super::InventoryRenderPlugin;
    pub use super::render::{RenderedItem, RenderingItem};
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

#[derive(Component, Deref)]
#[component(on_add = Self::on_add)]
#[relationship_target(relationship = RenderedSlot)]
pub struct RenderedInventory {
    #[deref]
    inventory: Handle<Inventory>,
    #[relationship]
    slots: Vec<Entity>,
}

impl Debug for RenderedInventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RenderedInventory")
            .field("inventory", &self.inventory)
            .finish()
    }
}

impl RenderedInventory {
    pub fn new(inventory: Handle<Inventory>) -> Self {
        Self { inventory, slots: Vec::new() }
    }

    pub fn get_slot(&self, index: usize) -> Option<Entity> {
        self.slots.get(index).cloned()
    }

    fn on_add(mut world: DeferredWorld, ctx: HookContext) {
        let target: AssetId<Inventory> = world.get::<RenderedInventory>(ctx.entity).expect("This is RenderInventory OnAdd").into();
        let mut inventorys = world.resource_mut::<Assets<Inventory>>();
        let Some(inventory) = inventorys.get_mut_untracked(target) else {
            warn!("Failed to find Inventory({:?}) for RenderedInventory({:?})", target, ctx.entity);
            return;
        };
        inventory.add_renderer(ctx.entity);
    }
}

impl Into<AssetId<Inventory>> for &RenderedInventory {
    fn into(self) -> AssetId<Inventory> {
        self.inventory.id()
    }
}

#[derive(Component, Debug)]
#[relationship(relationship_target = RenderedInventory)]
#[component(immutable)]
pub struct RenderedSlot {
    #[relationship]
    pub(crate) inventory: Entity,
    pub(crate) slot: CellType,
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