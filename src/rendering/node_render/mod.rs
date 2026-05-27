use bevy::prelude::*;

use crate::{prelude::*, rendering::{RenderedInventory, RenderedSlot}};

pub struct InventoryNodePlugin {
    pub auto_require: bool,
}

impl Plugin for InventoryNodePlugin {
    fn build(&self, app: &mut App) {
        if self.auto_require {
            app.register_required_components::<RenderedInventory, InventoryNode>();
        }
        app.add_systems(PreUpdate, (spawn_inventory_window, slot_node::update_image_cell_scale, item_node::update_item_node_image));
    }
}

mod inventory_node;
pub use inventory_node::InventoryNode;
mod slot_node;
pub use slot_node::SlotNode;
mod item_node;
pub use item_node::ItemNode;

fn spawn_inventory_window(
    mut commands: Commands,
    mut new: Populated<(Entity, &RenderedInventory, Option<&mut ImageNode>, &mut Node), Added<InventoryNode>>,
    mut inventory_manager: InventoryManager,
    styles: InventoryStyler,
) {
    for (entity, target, back_ground, mut node) in new {
        let style = styles.style(entity);
        node.display = Display::Grid;
        node.grid_auto_columns = GridTrack::px(style.cell_size.x);
        node.grid_auto_rows = GridTrack::px(style.cell_size.y);
        let Some(inventory) = inventory_manager.open_inventory(target) else {
            warn!("Failed to get Inventory({:?}) for Node({:?})", target.inventory, entity);
            continue;
        };
        if let Some(bkg) = style.background.clone() {
            if let Some(mut image) = back_ground {
                image.image = bkg;
            } else {
                commands.entity(entity).insert(ImageNode { image: bkg, ..Default::default() });
            }
        }
        let mut size_max = Vec2::ZERO;
        let mut size_min = Vec2::ZERO;
        for (slot_type, shape) in inventory.slots() {
            let size = shape.bounds().size().as_vec2() * style.cell_size;
            let pos = shape.offset.as_vec2() * style.cell_size;
            size_max = size_max.max(pos + size);
            size_min = size_min.min(pos);
            commands.spawn((
                SlotNode,
                Node {
                width: Val::Px(size.x),
                height: Val::Px(size.y),
                grid_row: GridPlacement::start_span(shape.offset.y as i16 + 1, shape.bounds().size().y as u16),
                grid_column: GridPlacement::start_span(shape.offset.x as i16 + 1, shape.bounds().size().x as u16),
                ..Default::default()
                },
                ImageNode {
                    image: style.cell_icon.clone(),
                    image_mode: NodeImageMode::Tiled { tile_x: true, tile_y: true, stretch_value: 1.0 },
                    ..Default::default()
                },
                ChildOf(entity),
                ZIndex(-1),
                RenderedSlot {
                    inventory: entity,
                    slot: slot_type.clone(),
                }
            ));
        }
        node.width = Val::Px(size_max.x - size_min.x);
        node.height = Val::Px(size_max.y - size_min.y);
        for (item, shape) in inventory.items() {
            commands.spawn((
                ItemNode(entity),
                RenderedItem {
                    item: *item,
                },
                Node {
                    grid_row: GridPlacement::start(shape.offset.y as i16 + 1),
                    grid_column: GridPlacement::start(shape.offset.x as i16 + 1),
                    ..Default::default()
                },
                ChildOf(entity),
            ));
        }
    }
}
