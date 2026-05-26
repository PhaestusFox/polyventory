use bevy::{
    ecs::query::{QueryData, ROQueryItem},
    prelude::*,
};

mod inventory;
#[cfg(feature = "rendering")]
mod mouse_interaction;
#[cfg(feature = "rendering")]
mod rendering;

pub mod prelude {
    pub use crate::inventory::{
        Cells, Entry, Inventory, Item, ItemDescriptor, Orientation, Shape, Slot,
        SlotType,
    };

    pub use crate::inventory::manager::{AddFailed, InventoryCommands, InventoryManager};

    #[cfg(feature = "rendering")]
    pub use crate::mouse_interaction::MouseInventoryPlugin;
    #[cfg(feature = "rendering")]
    pub use crate::mouse_interaction::{ToolTipPlugin, ToolTipSettings};
    #[cfg(feature = "rendering")]
    pub use crate::rendering::render_prelude::*;
}

#[cfg(feature = "rendering")]
pub use rendering::render_prelude;

#[cfg(feature = "rendering")]
pub use render_prelude::InventoryRenderPlugin;

pub struct PolyventoryPlugin;

impl Plugin for PolyventoryPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<inventory::Inventory>();
        app.init_asset::<inventory::ItemDescriptor>();
        app.init_asset::<inventory::InventoryDescriptor>();
        app.init_asset_loader::<inventory::ItemDescriptorLoader>();
        app.init_asset_loader::<inventory::InventoryDescriptorLoader>();

        app.register_type::<inventory::entry::Entry>();
    }
}

