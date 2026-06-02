use bevy::{ecs::system::SystemParam, prelude::*};
use crate::{inventory::entry::Entry, prelude::*};


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
        app.init_resource::<CursorInventory>();
    }
}

fn spawn_cursor_slot(mut commands: Commands, cursor: Res<CursorInventory>) {
    commands.spawn((
        CursorSlot,
        Transform::default(),
        Visibility::Visible,
        Name::new("Cursor Slot"),
        RenderedInventory::new(cursor.inventory.clone()),
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
    origin: Option<InventoryHandle>,
}

/// a handle to an inventory slot
/// used to keep track of where and item goes without searching everywhere
pub struct InventoryHandle {
    // the inventory the item came from
    pub(crate) inventory: AssetId<Inventory>,
    // the slot index inside the inventory the item came from
    pub(crate) entry: Entry,
}

#[derive(SystemParam)]
struct InventoryCursor<'w, 's> {
    commands: Commands<'w, 's>,
    cursor: Single<'w, 's, (Entity, Option<&'static HeldItem>), With<CursorSlot>>,
    inventory: ResMut<'w, CursorInventory>,
    manager: InventoryManager<'w, 's>,
}

impl InventoryCursor<'_, '_> {
    pub fn hold(&mut self, item: Entity, origin: Option<InventoryHandle>) {
        let inv: AssetId<Inventory> = self.inventory.as_ref().into();
        let mut inventory = self.manager.open_inventory(inv).expect("Cursor Inventory exists");
        if let Some(origin) = &origin {
            // add item with current shape
            inventory.add_item_at(item, origin.entry.shape.clone()).expect("all items should fit in an any slot");
        } else {
            // add item with default shape
            inventory.add_item(item).expect("all items should fit in an any slot");
        };
        self.inventory.origin = origin;
        self.commands.entity(item).insert(InInventory(self.inventory.as_ref().into(), CellType::Any));
    }

    pub fn entity(&self) -> Entity {
        self.cursor.0
    }

    pub fn is_empty(&self) -> bool {
        let inv = self.manager.read_inventory(self.inventory.as_ref()).expect("Cursor Inventory Always exists");
        inv.is_empty()
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

#[derive(Resource)]
pub struct CursorInventory {
    inventory: Handle<Inventory>,
    origin: Option<InventoryHandle>,
}

impl Into<AssetId<Inventory>> for &CursorInventory {
    fn into(self) -> AssetId<Inventory> {
        self.inventory.id()
    }
}

impl FromWorld for CursorInventory {
    fn from_world(world: &mut World) -> Self {
        let mut inventorys = world.resource_mut::<Assets<Inventory>>();
        let mut inventory = Inventory::new("CursorInventory");
        inventory.add_slot(CellType::Any, Shape {
            offset: IVec2::ZERO,
            orientation: Orientation::Deg0,
            layout: Layout::Rect { size: UVec2::new(1, 1) },
        });
        let inventory = inventorys.add(inventory);
        CursorInventory { inventory, origin: None }
    }
}