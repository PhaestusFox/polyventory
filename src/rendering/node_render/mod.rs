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
        app.add_systems(PreUpdate, (spawn_inventory_window, update_image_cell_scale, item_node::update_item_node_image));
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
    new: Populated<(Entity, &RenderedInventory), Added<InventoryNode>>,
    mut inventory_manager: InventoryManager,
    styles: InventoryStyler,
) {
    for (entity, node) in new {
        let Some(inventory) = inventory_manager.open_inventory(node) else {
            warn!("Failed to get Inventory({:?}) for Node({:?})", node.inventory, entity);
            continue;
        };
        let style = styles.style(entity);
        if let Some(bkg) = style.background.clone() {
            commands.entity(entity).insert(ImageNode {
                image: bkg,
                ..Default::default()
            });
        }
        for (i, slot) in inventory.slots().iter().enumerate() {
            commands.spawn((
                SlotNode,
                RenderedSlot { index: i, inventory: entity },
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Px(slot.size.x as f32 * style.cell_size.x),
                    height: Val::Px(slot.size.y as f32 * style.cell_size.y),
                    top: Val::Px(slot.position.y as f32 * style.cell_size.y),
                    left: Val::Px(slot.position.x as f32 * style.cell_size.x),
                    ..Default::default()
                },
                ImageNode {
                    image: style.cell_icon.clone(),
                    image_mode: NodeImageMode::Tiled { tile_x: true, tile_y: true, stretch_value: 1.0 },
                    ..Default::default()
                },
                ChildOf(entity),
                Name::new(format!("Slot {}", i)),
            )).with_children(|root| {
                for entry in &slot.entries {
                    let size = entry.shape.size().as_vec2() * style.cell_size;
                    root.spawn((
                        ItemNode(entity),
                        RenderedItem {
                            item: entry.entity,
                        },
                        Node {
                            width: Val::Px(size.x),
                            height: Val::Px(size.y),
                            ..Default::default()
                        },
                    ));
                }
            });
        }
    }
}

fn update_image_cell_scale(
    mut images: Populated<(&mut ImageNode, &RenderedSlot), Added<RenderedSlot>>,
    styles: InventoryStyler,
    image_assets: Res<Assets<Image>>,
) {
    for (mut image, slot) in &mut images {
        let style = styles.style(slot.inventory);
        let Some(image_asset) = image_assets.get(&image.image) else {
            warn!("Failed to get Image asset for image handle {:?}", image.image);
            continue;
        };
        let NodeImageMode::Tiled { stretch_value, .. } = &mut image.image_mode else {
            continue;
        };
        let size = image_asset.size();
        let scale = style.cell_size / size.as_vec2();
        let s = scale.x.min(scale.y);
        *stretch_value = s;
    }
}