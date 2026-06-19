use super::*;
use bevy::{input::mouse::AccumulatedMouseScroll, prelude::*, window::PrimaryWindow};

pub struct MouseInventoryPlugin;

impl Plugin for MouseInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(detect_pickup);
        app.add_observer(detect_drop);
        app.add_systems(First, follow_mouse);
        app.add_systems(Update, rotate_item);
    }
}

fn detect_pickup(
    click: On<Pointer<Click>>,
    icons: Query<&RenderedItem>,
    mut commands: Commands,
    manager: InventoryManager,
) {
    // skip if not Primary click
    if click.button != PointerButton::Primary {
        return;
    }
    let Ok(rendered_item) = icons.get(click.entity) else {
        return;
    };
    let Some(pos) = click.hit.position else {
        // if click does not have a position just return offset Zero
        commands.trigger(PickupItem {
            item: rendered_item.item,
            offset: IVec2::ZERO,
        });
        return;
    };
    let pos = pos.truncate() + 0.5;

    let Some(inventory_id) = manager.find_item(rendered_item.item) else {
        // if item is not in an inventory just return Offset Zero
        commands.trigger(PickupItem {
            item: rendered_item.item,
            offset: IVec2::ZERO,
        });
        return;
    };

    let Some(inventory) = manager.read_inventory(inventory_id) else {
        warn!(
            "Failed to open inventory {:?} for cursor drop",
            inventory_id
        );
        return;
    };
    let Some(size) = inventory.get_shape(rendered_item.item) else {
        warn!("Failed to find item in inventory");
        return;
    };

    let clicked = match size.orientation.intersection(Orientation::DEG270).bits() {
        0b01 => {
            let bounds = size.layout.bounds() * size.orientation;
            let p = (size.layout.size().as_vec2() * pos).as_ivec2();
            ivec2(-bounds.max.x + p.y, -bounds.min.y - p.x)
        }
        0b10 => {
            let bounds = size.layout.bounds();
            let p = (size.layout.size().as_vec2() * pos).as_ivec2();
            p - bounds.max
        }
        0b11 => {
            let bounds = size.layout.bounds() * size.orientation;
            let p = (size.layout.size().as_vec2() * pos).as_ivec2();
            ivec2(-bounds.min.x - p.y, -bounds.max.y + p.x)
        }
        _ => {
            let bounds = size.layout.bounds();
            let p = (size.layout.size().as_vec2() * pos).as_ivec2();
            bounds.min - p
        }
    };
    trace!(
        "Clicked on item entity {:?} with item {:?}: {} = {:?}",
        click.entity, rendered_item.item, pos, clicked
    );
    commands.trigger(PickupItem {
        item: rendered_item.item,
        offset: clicked,
    });
}

fn detect_drop(
    mut click: On<Pointer<Click>>,
    mut commands: Commands,
    mut cursor: InventoryCursor,
    icons: Query<&RenderedInventory>,
    slot: Query<&RenderedSlot>,
) {
    if !cursor.can_drop() {
        return;
    }
    let Ok(slot) = slot.get(click.entity) else {
        return;
    };
    let Some(pos) = click.hit.position else {
        warn!(
            "Click on inventory entity {:?} did not have a hit position",
            click.entity
        );
        return;
    };
    let pos = pos.truncate() + Vec2::splat(0.5);
    let Ok(inventory_id) = icons.get(slot.inventory) else {
        warn!(
            "Clicked inventory entity {:?} does not have a valid slot index",
            click.entity
        );
        return;
    };
    let Some(inventory) = cursor.manager.read_inventory(inventory_id) else {
        warn!(
            "Failed to open inventory {:?} for cursor drop",
            inventory_id
        );
        return;
    };
    let Some(size) = inventory.get_slot(&slot.slot) else {
        error!(
            "Clicked slot with type {} but inventory {:?} does not have a this type",
            slot.slot, inventory_id
        );
        return;
    };
    let clicked = (size.bounds().size().as_vec2() * pos).as_ivec2();

    let (item, mut shape) = cursor.last().expect("Cursor is not empty");
    shape.offset += clicked + size.offset;
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
    inv: InventoryCursor,
    styles: InventoryStyler,
) {
    let Some(pos) = window.cursor_position() else {
        return;
    };
    let style = styles.style(inv.entity());
    let mut x = pos.x - style.cell_size.x * 0.5;
    let mut y = pos.y - style.cell_size.y * 0.5;
    if let Some((_, shape)) = inv.last() {
        if shape.offset.x.is_negative() {
            x -= shape.offset.x.abs() as f32 * style.cell_size.x;
        }
        if shape.offset.y.is_negative() {
            y -= shape.offset.y.abs() as f32 * style.cell_size.y;
        }
    }
    cursor.translation = Val2 {
        x: Val::Px(x),
        y: Val::Px(y),
    }
}

fn rotate_item(
    mut commands: Commands,
    cursor: InventoryCursor,
    mouse_wheel: Res<AccumulatedMouseScroll>,
) {
    if cursor.is_empty() {
        return;
    }
    let rot = mouse_wheel.delta.y as i32 % 4;
    if rot.is_positive() {
        for _ in 0..rot {
            commands.trigger(super::OrientateItem::ClockWise);
        }
    } else if rot.is_negative() {
        for _ in 0..rot.abs() {
            commands.trigger(super::OrientateItem::CounterClockWise);
        }
    }
}
