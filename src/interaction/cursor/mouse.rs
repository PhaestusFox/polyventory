use bevy::{prelude::*, window::PrimaryWindow};
use super::*;

pub struct MouseInventoryPlugin;

impl Plugin for MouseInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(detect_pickup);
        app.add_observer(detect_drop);
        app.add_systems(First, follow_mouse);
    }
}

fn detect_pickup(
    click: On<Pointer<Click>>,
    icons: Query<&RenderedItem>,
    mut commands: Commands,
) {
    // skip if not Primary click
    if click.button != PointerButton::Primary {
        return;
    }
    let Ok(rendered_item) = icons.get(click.entity) else {
        return;
    };
    trace!("Clicked on item entity {:?} with item {:?}", click.entity, rendered_item.item);
    commands.trigger(PickupItem(rendered_item.item));
}

fn detect_drop(
    mut click: On<Pointer<Click>>,
    mut commands: Commands,
    mut cursor: InventoryCursor,
    icons: Query<&RenderedInventory>,
    slot: Query<&RenderedSlot>,
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
    let Some(inventory) = cursor.manager.read_inventory(inventory_id) else {
        warn!("Failed to open inventory {:?} for cursor drop", inventory_id);
        return;
    };
    let Some(size) = inventory.get_slot(&slot.slot) else {
        error!("Clicked slot with type {} but inventory {:?} does not have a this type", slot.slot, inventory_id);
        return;
    };
    let clicked = (size.bounds().size().as_vec2() * pos).as_ivec2();

    let (item, mut shape) = cursor.last().expect("Cursor is not empty");
    shape.offset = clicked;
    commands.trigger(DropItem {
        inventory: inventory_id.id(),
        item,
        shape,
        cell_type: slot.slot.clone(),
    });
}

fn follow_mouse(
    window: Single<&Window, With<PrimaryWindow>>,
    mut cursor: Single<&mut UiTransform, With<CursorSlot>>,
) {    
    let Some(pos) = window.cursor_position() else {
        return;
    };
    cursor.translation = Val2 { x: Val::Px(pos.x), y: Val::Px(pos.y) }
}