use crate::rendering::render::RenderingItem;

use super::*;

pub fn try_pickup(
    to: On<PickupItem>,
    mut commands: Commands,
    items: Query<(&Item, &RenderingItem)>,
    item_descriptors: Res<Assets<ItemDescriptor>>,
    mut inventory_manager: InventoryManager,
    mut cursor: InventoryCursor,
) {
    println!("Received PickupItem event for entity {:?}", to.0);
    let Ok((item, rendering)) = items.get(to.0) else {
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
    let Some(mut inventory) = inventory_manager.open_inventory(inventory_id) else {
        warn!("Inventory asset for item entity {:?} not found", to.0);
        return;
    };
    let Ok((slot, _)) = inventory.remove_item(to.0) else {
        warn!("Failed to remove item entity {:?} from inventory", to.0);
        return;
    };
    cursor.hold(to.0, InventoryHandle {
        inventory: inventory_id,
        slot_index: slot,
    });
    commands.entity(to.0).despawn_related::<RenderingItem>();
    println!("Picked up item {:?} from inventory {:?} slot {}", to.0, inventory_id, slot);
}