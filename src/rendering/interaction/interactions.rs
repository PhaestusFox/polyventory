use crate::rendering::render::RenderingItem;

use super::*;

pub fn try_pickup(
    to: On<PickupItem>,
    mut commands: Commands,
    items: Query<&Item>,
    item_descriptors: Res<Assets<ItemDescriptor>>,
    mut inventory_manager: InventoryManager,
    mut cursor: InventoryCursor,
) {
    if !cursor.is_empty() {
        warn!("Attempted to pick up item {:?} while already holding an item", to.0);
        return;
    }
    let Ok(item) = items.get(to.0) else {
        warn!("PickupItem event references an entity {:?} that does not have an Item component", to.0);
        return;
    };
    let Some(descriptor) = item_descriptors.get(item) else {
        warn!("Item entity {:?} has an Item component with a handle that does not correspond to an ItemDescriptor asset", to.0);
        return;
    };
    if !descriptor.is_moveable() {
        warn!("Item entity {:?} is not moveable", to.0);
        return;
    }
    let Some(inventory_id) = inventory_manager.find_item(to.0) else {
        warn!("Item entity {:?} is not in any inventory", to.0);
        return;
    };
    let s = inventory_manager.get_strong(inventory_id).unwrap();
    let Some(mut inventory) = inventory_manager.open_inventory(&s) else {
        warn!("Inventory asset for item entity {:?} not found", to.0);
        return;
    };
    let Ok((slot, _)) = inventory.remove_item(to.0) else {
        warn!("Failed to remove item entity {:?} from inventory", to.0);
        return;
    };
    cursor.hold(to.0, InventoryHandle {
        inventory: s.id(),
        slot_index: slot,
    });
    commands.entity(to.0).despawn_related::<RenderingItem>();
    println!("Picked up item {:?} from inventory {:?} slot {}", to.0, inventory_id, slot);
}

pub fn try_drop(
    to: On<DropItem>,
    mut cursor: InventoryCursor,
    mut inventory_manager: InventoryManager,
) {
    let Some(item) = cursor.item() else {
        return;
    };
    let Some(mut inventory) = inventory_manager.open_inventory(to.inventory) else {
        warn!("Inventory asset {:?} not found", to.inventory);
        return;
    };
    if let Err(e)  = inventory.add_item_at(item, to.pos, Orientation::Rot0) {
        warn!("Failed to add item entity {:?} to inventory {:?} at position {:?}: {:?}", item, to.inventory, to.pos, e);
        return;
    }
    cursor.drop();
}