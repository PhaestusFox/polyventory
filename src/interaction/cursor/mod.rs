use bevy::{ecs::system::SystemParam, prelude::*};
use crate::{interaction::{InventoryHandle, interactions::MoveItem}, prelude::*};

#[cfg(feature = "mouse_interaction")]
mod mouse;

#[cfg(feature = "rendering")]
mod visuals;

pub struct CursorPlugin;

impl Plugin for CursorPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CursorInventory>();
        #[cfg(feature = "rendering")]
        app.add_plugins(visuals::plugin);

        app.add_observer(try_pickup);
        app.add_observer(try_drop);

        #[cfg(feature = "mouse_interaction")]
        app.add_plugins(mouse::MouseInventoryPlugin);
    }
}

#[derive(SystemParam)]
pub struct InventoryCursor<'w, 's> {
    commands: Commands<'w, 's>,
    cursor: Single<'w, 's, Entity, With<CursorSlot>>,
    inventory: ResMut<'w, CursorInventory>,
    manager: InventoryManager<'w, 's>,
}

impl InventoryCursor<'_, '_> {
    pub fn hold(&mut self, item: Entity, origin: Option<InventoryHandle>) {
        self.inventory.origin = origin;
        // let inv: AssetId<Inventory> = self.inventory.as_ref().into();
        // let mut inventory = self.manager.open_inventory(inv).expect("Cursor Inventory exists");
        if let Some(origin) = &self.inventory.origin {
            let mut shape = origin.entry.clone();
            shape.offset = IVec2::ZERO;
            trace!("Picking up item: {:?}", shape);
            self.commands.trigger(MoveItem::InsertItem {
                item,
                inventory: self.inventory.inventory.id(),
                shape,
                cell_type: CellType::Any,
            });
            // add item with current shape
            // inventory.add_item_at(item, origin.entry.clone()).expect("all items should fit in an any slot");
        } else {
            // add item with default shape
            // inventory.add_item(item).expect("all items should fit in an any slot");
            self.commands.trigger(MoveItem::FitItem {
                item,
                inventory: self.inventory.inventory.id(),
            });
        };
    }

    pub fn entity(&self) -> Entity {
        *self.cursor
    }

    pub fn is_empty(&self) -> bool {
        let inv = self.manager.read_inventory(self.inventory.as_ref()).expect("Cursor Inventory Always exists");
        inv.is_empty()
    }

    pub fn last(&self) -> Option<(Entity, Shape)> {
        let inv = self.manager.read_inventory(self.inventory.as_ref()).expect("Cursor Inventory Always exists");
        inv.items().last().map(|(e, s)| (*e, s.shape.clone()))
    }

    pub fn set_style(&mut self, style: Handle<InventoryStyle>) {
        self.commands.entity(self.entity()).insert(InventoryStyleHandle(style));
    }
}

#[derive(Resource)]
pub struct CursorInventory {
    pub inventory: Handle<Inventory>,
    pub origin: Option<InventoryHandle>,
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

#[derive(Component)]
/// A marker for the entity that is set under the cursor when it holds and item
pub struct CursorSlot;

#[derive(Event)]
/// Attempt to move this item into the cursor inventory, removing it from its current inventory if successful
pub struct PickupItem(pub Entity);

#[derive(Event)]
/// Attempt to move this item from the cursor inventory into the specified inventory, removing it from the cursor if successful
pub struct DropItem {
    pub inventory: AssetId<Inventory>,
    pub item: Entity,
    pub shape: Shape,
    pub cell_type: CellType,
}

pub fn try_pickup(
    to: On<PickupItem>,
    mut commands: Commands,
    items: Query<&Item>,
    item_descriptors: Res<Assets<ItemDescriptor>>,
    mut cursor: InventoryCursor,
) {
    if !cursor.is_empty() {
        warn!("Attempted to pick up item {:?} while already holding an item", to.0);
        return;
    }
    let Ok(item) = items.get(to.0) else {
        warn!("PickupItem event references an entity({:?}) that is not an item", to.0);
        return;
    };
    // TODO add locked marker component so items can be locked in an inventory and not be picked up


    let origin: Option<InventoryHandle>;
    if let Some(find) = cursor.manager.find_item(to.0)
    && let Some(inventory) = cursor.manager.read_inventory(find)
    && let Some(entry) = inventory.get_shape(to.0) {
        origin = Some(InventoryHandle {
            inventory: find,
            entry: entry.clone(),
        });
    } else {
        origin = None;
    }

    cursor.hold(to.0, origin);
    trace!("Picked up item {:?}", to.0);
}

pub fn try_drop(
    to: On<DropItem>,
    mut commands: Commands,
    cursor: InventoryCursor,
) {
    trace!("Attempting to drop item {:?} into inventory {:?} with shape {:?}", to.item, to.inventory, to.shape);
    let Some(inv) = cursor.manager.read_inventory(to.inventory) else {
        warn!("Failed to open inventory {:?} for cursor drop", to.inventory);
        return;
    };
    if inv.can_fit(&to.cell_type, &to.shape) {
        commands.trigger(MoveItem::PlaceItem {
            item: to.item,
            inventory: to.inventory,
            shape: to.shape.clone(),
            cell_type: to.cell_type.clone()
        });
    } else {
        warn!("Item with shape {:?} does not fit in inventory {:?} slot type {:?}", to.shape, to.inventory, to.cell_type);
        return;
    }
}
