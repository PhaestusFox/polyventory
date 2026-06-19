use crate::inventory::Operations;
use crate::{
    interaction::{InventoryHandle, interactions::MoveItem},
    prelude::*,
};
use bevy::{ecs::system::SystemParam, prelude::*};

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
        app.add_observer(try_orientate);

        #[cfg(feature = "mouse_interaction")]
        app.add_plugins(mouse::MouseInventoryPlugin);

        app.add_systems(First, clear_picking);
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
    pub fn hold(&mut self, item: Entity, origin: Option<InventoryHandle>, offset: IVec2) {
        self.inventory.picked = true;
        self.inventory.origin = origin;
        // let inv: AssetId<Inventory> = self.inventory.as_ref().into();
        // let mut inventory = self.manager.open_inventory(inv).expect("Cursor Inventory exists");
        if let Some(origin) = &self.inventory.origin {
            let mut shape = origin.entry.clone();
            shape.offset = offset;
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

    pub fn update(&mut self, item: Entity, orientation: Orientation) {
        let mut inv = self
            .manager
            .open_inventory(&self.inventory.inventory)
            .expect("Cursor Inventory Always Exists");
        let Some(shape) = inv.get_shape_mut(item) else {
            trace!("Attempted to update item no in hand");
            return;
        };
        let offset = calc_offset(shape, orientation);
        shape.orientation = orientation;
        shape.offset = offset;
    }

    pub fn entity(&self) -> Entity {
        *self.cursor
    }

    pub fn is_empty(&self) -> bool {
        let inv = self
            .manager
            .read_inventory(self.inventory.as_ref())
            .expect("Cursor Inventory Always exists");
        inv.is_empty()
    }

    pub fn can_drop(&self) -> bool {
        !self.inventory.picked && !self.is_empty()
    }

    pub fn last(&self) -> Option<(Entity, Shape)> {
        let inv = self
            .manager
            .read_inventory(self.inventory.as_ref())
            .expect("Cursor Inventory Always exists");
        inv.items().last().map(|(e, s)| (*e, s.shape.clone()))
    }

    pub fn set_style(&mut self, style: Handle<InventoryStyle>) {
        self.commands
            .entity(self.entity())
            .insert(InventoryStyleHandle(style));
    }
}

#[derive(Resource)]
pub struct CursorInventory {
    pub inventory: Handle<Inventory>,
    pub origin: Option<InventoryHandle>,
    pub picked: bool,
}

impl Into<AssetId<Inventory>> for &CursorInventory {
    fn into(self) -> AssetId<Inventory> {
        self.inventory.id()
    }
}

impl FromWorld for CursorInventory {
    fn from_world(world: &mut World) -> Self {
        let mut inventorys = world.resource_mut::<Assets<Inventory>>();
        let inventory = Inventory::new("CursorInventory");
        let inventory = inventorys.add(inventory);
        CursorInventory {
            inventory,
            origin: None,
            picked: false,
        }
    }
}

#[derive(Component)]
/// A marker for the entity that is set under the cursor when it holds and item
pub struct CursorSlot;

#[derive(Event)]
/// Attempt to move this item into the cursor inventory, removing it from its current inventory if successful
pub struct PickupItem {
    item: Entity,
    offset: IVec2,
}

pub fn try_pickup(
    to: On<PickupItem>,
    mut commands: Commands,
    items: Query<&Item>,
    item_descriptors: Res<Assets<ItemDescriptor>>,
    mut cursor: InventoryCursor,
) {
    if !cursor.is_empty() {
        warn!(
            "Attempted to pick up item {:?} while already holding an item",
            to.item
        );
        return;
    }
    let Ok(item) = items.get(to.item) else {
        warn!(
            "PickupItem event references an entity({:?}) that is not an item",
            to.item
        );
        return;
    };
    // TODO add locked marker component so items can be locked in an inventory and not be picked up

    let origin: Option<InventoryHandle>;
    if let Some(find) = cursor.manager.find_item(to.item)
        && let Some(inventory) = cursor.manager.read_inventory(find)
        && let Some(entry) = inventory.get_shape(to.item)
    {
        origin = Some(InventoryHandle {
            inventory: find,
            entry: entry.clone(),
        });
    } else {
        origin = None;
    }

    cursor.hold(to.item, origin, to.offset);
    trace!("Picked up item {:?}", to.item);
}

#[derive(Event)]
/// Attempt to move this item from the cursor inventory into the specified inventory, removing it from the cursor if successful
pub struct DropItem {
    pub inventory: AssetId<Inventory>,
    pub item: Entity,
    pub shape: Shape,
    pub cell_type: CellType,
}

pub fn try_drop(to: On<DropItem>, mut commands: Commands, cursor: InventoryCursor) {
    trace!(
        "Attempting to drop item {:?} into inventory {:?} with shape {:?}",
        to.item, to.inventory, to.shape
    );
    let Some(inv) = cursor.manager.read_inventory(to.inventory) else {
        warn!(
            "Failed to open inventory {:?} for cursor drop",
            to.inventory
        );
        return;
    };
    if inv.can_fit(&to.cell_type, &to.shape) {
        commands.trigger(MoveItem::PlaceItem {
            item: to.item,
            inventory: to.inventory,
            shape: to.shape.clone(),
            cell_type: to.cell_type.clone(),
        });
    } else {
        warn!(
            "Item with shape {:?} does not fit in inventory {:?} slot type {:?}",
            to.shape, to.inventory, to.cell_type
        );
        return;
    }
}

#[derive(Event)]
/// Attempt to rotate the item in the cursor inventory
pub enum OrientateItem {
    ClockWise,
    CounterClockWise,
}

pub fn try_orientate(event: On<OrientateItem>, mut cursor: InventoryCursor) {
    if cursor.is_empty() {
        trace!("Cursor is empty");
        return;
    }
    let Some((item, shape)) = cursor.last() else {
        error!("Cursor does not have `last` item when it is not empty this is a bug");
        return;
    };
    let new = match event.event() {
        OrientateItem::ClockWise => shape.orientation.operation(Operations::RotateClockWise),
        OrientateItem::CounterClockWise => shape
            .orientation
            .operation(Operations::RotateCounterClockWise),
    };
    cursor.update(item, new);
}

fn clear_picking(mut cursor: ResMut<CursorInventory>) {
    cursor.picked = false;
}

fn calc_offset(shape: &Shape, orientation: Orientation) -> IVec2 {
    let p = match shape.orientation.intersection(Orientation::DEG270).bits() {
        0b01 => {
            let bounds = shape.layout.bounds() * shape.orientation;
            // shape.offset = ivec2(-bounds.max.x + p.y, -bounds.min.y - p.x)
            // u = shape.offset.x
            // u = -bounds.max.x + p.y
            // p.y = u + bounds.max.x
            // v = shape.offset.y
            // v = -bounds.min.y - p.x
            // v + bounds.min.y = -p.x
            // p.x = -(v + bounds.min.y)
            ivec2(
                -(shape.offset.y + bounds.min.y),
                shape.offset.x + bounds.max.x,
            )
        }
        0b10 => {
            let bounds = shape.layout.bounds();
            shape.offset + bounds.max
        }
        0b11 => {
            let bounds = shape.layout.bounds() * shape.orientation;
            // shape.offset = ivec2(-bounds.min.x - p.y, -bounds.max.y + p.x)
            // u = shape.offset.x
            // u = -bounds.min.x - p.y
            // -p.y = u + bounds.min.x
            // v = shape.offset.y
            // v = -bounds.max.y + p.x
            // p.x = v + bounds.max.y
            ivec2(
                shape.offset.y + bounds.max.y,
                -(shape.offset.x + bounds.min.x),
            )
        }
        _ => {
            let bounds = shape.layout.bounds();
            -(shape.offset - bounds.min)
        }
    };

    match orientation.intersection(Orientation::DEG270).bits() {
        0b01 => {
            let bounds = shape.layout.bounds() * orientation;
            ivec2(-bounds.max.x + p.y, -bounds.min.y - p.x)
        }
        0b10 => {
            let bounds = shape.layout.bounds();
            p - bounds.max
        }
        0b11 => {
            let bounds = shape.layout.bounds() * orientation;
            ivec2(-bounds.min.x - p.y, -bounds.max.y + p.x)
        }
        _ => {
            let bounds = shape.layout.bounds();
            bounds.min - p
        }
    }
}

#[test]
fn test_calc_offset() {
    let mut shape = Shape {
        offset: IVec2::ZERO,
        orientation: Orientation::empty(),
        layout: Layout::Rect {
            size: UVec2::new(2, 3),
        },
    };
    for x in -10..=10 {
        for y in -10..=10 {
            shape.offset = ivec2(x, y);
            for orientation in Orientation::iter_orientations() {
                shape.orientation = orientation;
                assert_eq!(
                    calc_offset(&shape, orientation),
                    ivec2(x, y),
                    "{}",
                    orientation
                );
            }
        }
    }
}
