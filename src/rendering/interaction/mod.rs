use bevy::{ecs::system::SystemParam, prelude::*};
use crate::prelude::*;


#[cfg(feature = "node_rendering")]
pub mod node_interaction;

mod mouse;

mod interactions;

pub struct InteractionPlugin;

impl Plugin for InteractionPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(feature = "mouse_interaction")]
        app.add_plugins(mouse::MouseInventoryPlugin);
        app.add_observer(interactions::try_pickup);
        app.add_observer(interactions::try_drop);
        app.add_systems(Startup, spawn_cursor_slot);
    }
}

fn spawn_cursor_slot(mut commands: Commands) {
    commands.spawn((
        CursorSlot,
        Transform::default(),
        Visibility::Visible,
        Name::new("Cursor Slot"),
    ));
}

#[derive(Event)]
pub struct PickupItem(pub Entity);

#[derive(Event)]
pub struct DropItem {
    pub inventory: AssetId<Inventory>,
    pub pos: IVec2,
}

#[derive(Component)]
/// A marker for the entity that is set under the cursor when it holds and item
pub struct CursorSlot;

#[derive(Component)]
pub struct HeldItem {
    // The actual item entity not the rendered item entity
    item_entity: Entity,
    // the offset into the rendered item the cursor was so we can keep the cursor in the same relative position
    offset: Vec2,
    // where the item came from so we can return it if we drop it somewhere invalid
    origin: InventoryHandle
}

/// a handle to an inventory slot
/// used to keep track of where and item goes without searching everywhere
pub struct InventoryHandle {
    // the inventory the item came from
    pub(crate) inventory: AssetId<Inventory>,
    // the slot index inside the inventory the item came from
    pub(crate) slot_index: usize,
}

#[derive(SystemParam)]
struct InventoryCursor<'w, 's> {
    commands: Commands<'w, 's>,
    cursor: Single<'w, 's, (Entity, Option<&'static HeldItem>), With<CursorSlot>>,
}

impl InventoryCursor<'_, '_> {
    pub fn hold(&mut self, item: Entity, origin: InventoryHandle) {
        let cursor_entity = self.entity();
        self.commands.entity(cursor_entity).insert(HeldItem {
            item_entity: item,
            offset: Vec2::ZERO,
            origin,
        });
        #[cfg(feature = "node_rendering")]
        self.commands.entity(cursor_entity).insert(
            (super::render::RenderedItem {
            item: item,
        }, super::node_render::ItemNode(cursor_entity)));
    }

    pub fn entity(&self) -> Entity {
        self.cursor.0
    }

    pub fn is_empty(&self) -> bool {
        self.cursor.1.is_none()
    }
    
    pub fn item(&self) -> Option<Entity> {
        self.cursor.1.map(|rendered| rendered.item_entity)
    }

    pub fn drop(&mut self) {
        self.commands.entity(self.entity()).remove::<HeldItem>();
        #[cfg(feature = "node_rendering")]
        self.commands.entity(self.entity()).remove::<(super::render::RenderedItem, super::node_render::ItemNode, ImageNode)>();
    }
}