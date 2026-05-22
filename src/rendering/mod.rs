use bevy::prelude::*;

mod render;
#[cfg(feature = "ui_node")]
pub mod node_render;
mod style;

pub(crate) mod render_prelude {
    pub use super::InventoryRenderPlugin;
    pub use super::render::{DisplayedItem, SpawnInventory, InventorySlot};
    pub use super::style::*;
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