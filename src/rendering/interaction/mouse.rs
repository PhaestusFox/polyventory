use bevy::ecs::system::{SystemParam};
use bevy::prelude::*;
use crate::prelude::*;
use super::*;

pub struct MouseInventoryPlugin;

impl Plugin for MouseInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(detect_pickup);
        app.add_observer(detect_drop);
    }
}

fn detect_pickup(
    click: On<Pointer<Click>>,
    icons: Query<&RenderedItem>,
    mut commands: Commands,
) {
    let Ok(rendered_item) = icons.get(click.entity) else {
        return;
    };
    println!("Clicked on item entity {:?} with item {:?}", click.entity, rendered_item.item);
    commands.trigger(PickupItem(rendered_item.item));
}

fn detect_drop(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    cursor: InventoryCursor,
    icons: Query<&RenderedInventory>,
    slot: Query<&RenderedSlot>,
    mut inventory_manager: InventoryManager,
) {
    if cursor.is_empty() {
        return;
    }
    let Ok(slot) = slot.get(click.entity) else {
        return;
    };
    let Some(pos) = click.hit.position else {
        warn!("Click on inventory entity {:?} did not have a hit position", click.entity);
        return;
    };
    let pos = pos.truncate() + Vec2::splat(0.5);
    let Ok(inventory_id) = icons.get(slot.inventory) else {
        warn!("Clicked inventory entity {:?} does not have a valid slot index", click.entity);
        return;
    };
    let Some(inventory) = inventory_manager.open_inventory(inventory_id) else {
        warn!("Clicked inventory entity {:?} does not correspond to an open inventory", click.entity);
        return;
    };
    let Some(slot) = inventory.get_slot(slot.index) else {
        warn!("Clicked inventory entity {:?} has an invalid slot index", click.entity);
        return;
    };
    let clicked_in = (pos * slot.size.as_vec2()).as_ivec2() + slot.position;
    commands.trigger(DropItem {
        inventory: inventory_id.into(),
        pos: clicked_in,
    });
    println!("Clicked on inventory entity {:?} at position {:?} to drop item", click.entity, clicked_in);
}