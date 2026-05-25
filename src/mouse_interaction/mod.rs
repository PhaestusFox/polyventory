#[cfg(feature = "tooltips")]
mod tooltip;
#[cfg(feature = "tooltips")]
pub use tooltip::{ToolTipPlugin, ToolTipSettings};

use crate::{
    inventory::{Inventory, Item, ItemDescriptor, Orientation, Shape},
    mouse_interaction::tooltip::ToolTipAction, rendering::RenderedInventory,
};

use bevy::{ecs::system::SystemParam, input::mouse::AccumulatedMouseMotion, prelude::*, window::PrimaryWindow};

use crate::prelude::*;

pub struct MouseInventoryPlugin;

impl Plugin for MouseInventoryPlugin {
    fn build(&self, app: &mut App) {
        // app.add_observer(pickup_item);
        app.add_observer(drop_item);
        app.add_observer(rotate_held_item);
        app.add_systems(
            PostUpdate,
            update_hand.run_if(|hand: Hand| hand.held_item.is_some()),
        );
    }
}

#[derive(SystemParam)]
pub struct Hand<'w> {
    held_item: Option<Res<'w, HeldItem>>,
}

#[derive(SystemParam)]
pub struct HandMut<'w> {
    held_item: Option<ResMut<'w, HeldItem>>,
}

#[derive(Resource)]
struct HeldItem {
    entity: Entity,
    offset: Vec2,
    origin: (AssetId<Inventory>, Entry, usize),
}

fn pickup_item(
    click: On<Pointer<Click>>,
    mut commands: Commands,
    hand: Hand,
    inventorys: Query<&RenderedInventory>,
    mut inventory_manager: InventoryManager,
    mut items: Query<(Entity, &ChildOf, &RenderedItem, AnyOf<(&GlobalTransform, &UiGlobalTransform)>)>,
    mut slots: Query<&RenderedSlot>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    if hand.held_item.is_some() {
        if inventorys.contains(click.entity) {
            commands.trigger(DropItem {
                inventory: click.entity,
                position: IVec2::new(4, 6),
            });
        }
        return;
    }
    let Ok((icon, slot, item, pos)) = items.get_mut(click.entity) else {
        return;
    };
    let Ok(slot_render) = slots.get(slot.0) else {
        warn!("Clicked Item is not in an inventory slot");
        return;
    };
    let Ok(inventory_handle) = inventorys.get(slot_render.inventory) else {
        warn!("Clicked inventory not found");
        return;
    };
    let Some(mut inventory) = inventory_manager.open_inventory(inventory_handle) else {
        warn!("Failed to open inventory {:?}", slot_render.inventory);
        return;
    };

    let (slot, entry) = match inventory.remove_item(item.item) {
        Ok(entry) => entry,
        Err(e) => {
            warn!("Failed to remove item from inventory: {:?}", e);
            return;
        }
    };

    let offset = match pos {
        (_, Some(ui)) => {
            window.cursor_position().unwrap_or_default() * window.scale_factor() - ui.translation
        },
        (Some(global), None) => {
            global.translation().truncate() - click.pointer_location.position
        },
        (None, None) => Vec2::ZERO,
    };

    commands
        .entity(icon)
        .remove::<ChildOf>()
        .remove::<Pickable>();
    commands.insert_resource(HeldItem {
        entity: icon,
        offset,
        origin: (inventory_handle.into(), entry, slot),
    });
}

#[derive(Event)]
struct DropItem {
    inventory: Entity,
    position: IVec2,
}

fn drop_item(
    click: On<DropItem>,
    mut commands: Commands,
    hand: Hand,
    inventorys: Query<&RenderedInventory>,
    mut icons: Query<(&RenderedItem, &Shape)>,
    mut inventory_manager: InventoryManager,
) {
    println!("Trying to drop item into inventory");
    let Some(held) = hand.held_item else {
        trace!("No item held, ignoring drop");
        return;
    };
    let Ok(inventory_render) = inventorys.get(click.inventory) else {
        trace!("Inventory({:?}) not found, ignoring drop", click.inventory);
        return;
    };
    let Some(mut inventory) = inventory_manager.open_inventory(inventory_render) else {
        trace!("Inventory asset not found, ignoring drop");
        return;
    };
    let Ok((icon, shape)) = icons.get_mut(held.entity) else {
        trace!("Item not found, ignoring drop");
        return;
    };

    match inventory.add_item_at(icon.item, click.position, shape.orientation) {
        Err(e) => {
            warn!("Failed to add item to inventory: {:?}", e);
            return;
        }
        Ok((slot, shape)) => {
            info!(
                "Added Item to inventory at position {:?} in slot {}",
                shape.position, slot
            );
            if let Some(slot) = inventory_render.get_slot(slot) {
                commands
                    .entity(held.entity)
                    .insert((ChildOf(slot), shape, Pickable::default()));
            } else {
                warn!("Added item to inventory but failed to find slot entity to parent to");
            }
            commands.remove_resource::<HeldItem>();
        }
    }

    // let Ok(item) = items.get(icon.entity) else {
    //     trace!("Item not found, ignoring drop");
    //     return;
    // };
    // let Some(descriptor) = descriptors.get(item.descriptor.id()) else {
    //     trace!("Item descriptor not found, ignoring drop");
    //     return;
    // };
    // match inventory.add_item_at(descriptor, held.entity, click.position, shape.orientation) {
    //     Ok((slot, shape)) => {
    //         info!("Added {} to inventory at position {:?} in slot {}", descriptor.name(), shape.position, slot);
    //         if let Some(slot) = inventory_render.get_slot(slot) {
    //             commands.entity(held.entity).insert((
    //                 ChildOf(slot),
    //                 shape,
    //                 Pickable::default(),
    //             ));
    //         } else {
    //             warn!("Added item to inventory but failed to find slot entity to parent to");
    //         }
    //         commands.remove_resource::<HeldItem>();
    //     },
    //     Err(errors) => {
    //         warn!("Failed to add item to inventory: {:?}\n{},{},{:?}", errors, descriptor.name(), click.position, shape.orientation);
    //     }
    // }
}

fn update_hand(
    hand: Hand,
    mut items: Query<(Option<&mut Transform>, Option<&mut UiTransform>)>,
    window: Single<&Window, With<PrimaryWindow>>,
) {
    let Some(item) = hand.held_item else {
        return;
    };
    let Ok((transform, ui_transform)) = items.get_mut(item.entity) else {
        return;
    };
    // todo add back sprite offset
    // if let Some(mut transform) = transform {
    //     transform.translation += mouse_moved.delta.extend(0.);
    // }
    if let (Some(mut ui_transform), Some(mouse)) = (ui_transform, window.cursor_position()) {
        let offset= item.offset + mouse;
        if let Val::Px(_) = ui_transform.translation.x {
            ui_transform.translation.x = Val::Px(offset.x);
        }
        if let Val::Px(_) = ui_transform.translation.y {
            ui_transform.translation.y = Val::Px(offset.y);
        }
    }
}

fn rotate_held_item(scroll: On<Pointer<Scroll>>, hand: Hand, mut icons: Query<&mut Shape>) {
    let Some(held) = hand.held_item else {
        return;
    };
    let Ok(mut shape) = icons.get_mut(held.entity) else {
        return;
    };
    shape.orientation = shape.orientation.rotate_by_scroll(scroll.y);
}

fn canvas_to_world(window: &Window) -> Vec2 {
    let half_size = window.size() / 2.;
    window
        .cursor_position()
        .map(|v| Vec2::new(v.x, -v.y) - vec2(half_size.x, -half_size.y))
        .unwrap_or_default()
}