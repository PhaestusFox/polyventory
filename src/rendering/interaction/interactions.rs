use crate::rendering::render::RenderingItem;

use super::*;

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
    && let Some(mut inventory) = cursor.manager.open_inventory(find)
    && let Some(entry) = inventory.remove(to.0) {
        origin = Some(InventoryHandle {
            inventory: find,
            entry,
        });
    } else {
        origin = None;
    }
    commands.entity(to.0).despawn_related::<RenderingItem>();

    cursor.hold(to.0, origin);
    println!("Picked up item {:?}", to.0);
}

pub fn try_drop(
    to: On<DropItem>,
    mut cursor: InventoryCursor,
) {
    // let Some(item) = cursor.item() else {
    //     return;
    // };
    // let Some(mut inventory) = inventory_manager.open_inventory(to.inventory) else {
    //     warn!("Inventory asset {:?} not found", to.inventory);
    //     return;
    // };
    // if let Err(e)  = inventory.add_item_at(item, to.pos, Orientation::Rot0) {
    //     warn!("Failed to add item entity {:?} to inventory {:?} at position {:?}: {:?}", item, to.inventory, to.pos, e);
    //     return;
    // }
    // cursor.drop();
}