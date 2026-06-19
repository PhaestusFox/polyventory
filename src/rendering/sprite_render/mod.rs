pub use crate::rendering::render_prelude::*;
pub use bevy::prelude::*;

pub struct InventorySpritePlugin {
    pub auto_require: bool,
}

impl Plugin for InventorySpritePlugin {
    fn build(&self, app: &mut App) {
        if self.auto_require {
            app.register_required_components::<RenderedInventory, InventorySprite>();
        }
    }
}

#[derive(Component, Default)]
pub struct InventorySprite;
