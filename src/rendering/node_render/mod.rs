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
    mut new: Populated<(Entity, &RenderedInventory, Option<&mut ImageNode>), Added<InventoryNode>>,
    mut inventory_manager: InventoryManager,
    styles: InventoryStyler,
) {
    for (entity, node, back_ground) in new {
        let Some(inventory) = inventory_manager.open_inventory(node) else {
            warn!("Failed to get Inventory({:?}) for Node({:?})", node.inventory, entity);
            continue;
        };
        let style = styles.style(entity);
        if let Some(bkg) = style.background.clone() {
            if let Some(mut image) = back_ground {
                image.image = bkg;
            } else {
                commands.entity(entity).insert(ImageNode { image: bkg, ..Default::default() });
            }
        }
        for (slot_type, shape) in inventory.slots() {
            let size = shape.bounds().size() * style.cell_size;
            let pos = shape.offset.as_vec2() * style.cell_size;
            commands.spawn((
                SlotNode,
                Node {
                position_type: PositionType::Absolute,
                width: Val::Px(size.x),
                height: Val::Px(size.y),
                top: Val::Px(pos.y),
                left: Val::Px(pos.x),
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
        for (item, shape) in inventory.items() {
            let size = shape.bounds().size() * style.cell_size;
            commands.spawn((
                ItemNode(entity),
                RenderedItem {
                    item: *item,
                },
                Node {
                    width: Val::Px(size.x),
                    height: Val::Px(size.y),
                    ..Default::default()
                },
                ChildOf(entity),
            ));
        }
    }
}
