use bevy::{platform::collections::HashSet, prelude::*};

use crate::{prelude::*, rendering::{RenderedInventory, RenderedSlot}};

pub struct InventoryNodePlugin {
    pub auto_require: bool,
}

impl Plugin for InventoryNodePlugin {
    fn build(&self, app: &mut App) {
        if self.auto_require {
            app.register_required_components::<RenderedInventory, InventoryNode>();
        }
        app.add_systems(PreUpdate, (spawn_inventory_node, update_inventory_node));
        app.add_systems(Update, (slot_node::update_image_cell_scale, item_node::update_item_node_image, style_inventory_node));
    }
}

mod inventory_node;
pub use inventory_node::InventoryNode;
mod slot_node;
pub use slot_node::SlotNode;
mod item_node;
pub use item_node::ItemNode;
pub use item_node::ItemNodes;

fn spawn_inventory_node(
    mut commands: Commands,
    new: Populated<(Entity, &RenderedInventory, Option<&mut ImageNode>, &mut Node, Option<&Pickable>), Added<InventoryNode>>,
    inventory_manager: InventoryManager,
    styles: InventoryStyler,
) {
    for (entity, target, back_ground, mut node, pickable) in new {
        let style = styles.style(entity);
        node.display = Display::Grid;
        node.grid_auto_columns = GridTrack::px(style.cell_size.x);
        node.grid_auto_rows = GridTrack::px(style.cell_size.y);
        let Some(inventory) = inventory_manager.read_inventory(target) else {
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
            let size = shape.layout.size();
            let a_size = shape.bounds().size().as_vec2() * style.cell_size;
            let pos = shape.offset.as_vec2() * style.cell_size;
            size_max = size_max.max(pos + a_size);
            size_min = size_min.min(pos);
            let mut offset = shape.offset;
            if offset.x >= 0 {
                offset.x += 1;
            }
            if offset.y >= 0 {
                offset.y += 1;
            }
            let mut e = commands.spawn((
                SlotNode,
                Node {
                    width: Val::Percent(100.),
                    height: Val::Percent(100.),
                    grid_row: GridPlacement::start_span(offset.y as i16, size.y as u16),
                    grid_column: GridPlacement::start_span(offset.x as i16, size.x as u16),
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
            if let Some(p) = pickable {
                e.insert(p.clone());
            }
        }
        // node.width = Val::Px(size_max.x - size_min.x);
        // node.height = Val::Px(size_max.y - size_min.y);
        for (item, shape) in inventory.items() {
            let size = shape.layout.size();
            let mut offset = shape.offset;
            if offset.x >= 0 {
                offset.x += 1;
            }
            if offset.y >= 0 {
                offset.y += 1;
            }
            let mut e = commands.spawn((
                ItemNode(entity),
                RenderedItem {
                    item: *item,
                },
                ChildOf(entity),
            ));
            if let Some(p) = pickable {
                e.insert(p.clone());
            }
        }
    }
}

fn update_inventory_node(
    mut commands: Commands,
    mut changes: MessageReader<AssetEvent<Inventory>>,
    mut inventory_nodes: Query<(Ref<RenderedInventory>, &mut Node, Option<&ItemNodes>, Option<&Pickable>), With<InventoryNode>>,
    slots: Query<&RenderedSlot, With<SlotNode>>,
    assets: Res<Assets<Inventory>>,
    render_nodes: Query<&RenderedItem>,
    styles: InventoryStyler,
) {
    let mut done = HashSet::new();
    for message in changes.read() {
        // Only care about modified events, added and removed should be handled by spawn and despawn systems
        let id = match message {
            AssetEvent::Modified { id } => id,
            _ => continue,
        };
        if done.contains(id) {
            continue;
        }
        done.insert(id);
        trace!("Inventory {} changed updating", id);

        // get the inventory that was modified
        let Some(inventory) = assets.get(*id) else {
            warn!("Received modified event for Inventory asset {:?} that is not currently loaded", id);
            continue;
        };
        for rendering in inventory.windows() {
            let Ok((root, mut node, item_nodes, picicking)) = inventory_nodes.get_mut(rendering) else {
                // TODO dont warn as this is just the case when sprite and node are both used
                warn!("Inventory asset {:?} modified but failed to find RenderedInventory component for entity {:?}", id, rendering);
                continue;
            };
            if root.is_added() {
                trace!("Skipping {} as it was only just added", rendering);
                continue;
            }
            let mut need = inventory.slots().map(|(k, _)| k).collect::<HashSet<_>>();
            let mut update = Vec::new();
            let mut remove = root.slots.iter().cloned().collect::<HashSet<_>>();
            for slot_entity in root.iter() {
                let Ok(slot) = slots.get(slot_entity) else {
                    // TODO dont warn as this is just the case when sprite and node are both used
                    warn!("Failed to get SlotNode component for entity {:?} while updating inventory node for Inventory asset {:?}", slot_entity, id);
                    // if its not ours dont kill it
                    remove.remove(&slot_entity);
                    continue;
                };
                // if we need this slot remove it from the to spawn list
                if need.remove(&slot.slot) {
                    // mark it for update so we can update its size
                    update.push((slot_entity, &slot.slot));
                    // remove it from the remove list so we dont delete it
                    remove.remove(&slot_entity);
                }
            }
            // despawn removed slots
            for slot in remove {
                warn!("Despawning slot entity {:?} for removed slot in Inventory asset {:?}", slot, id);
                commands.entity(slot).despawn();
            }

            let style = styles.style(rendering);
            let mut size_max = Vec2::ZERO;
            let mut size_min = Vec2::ZERO;
            for new in need {
                let shape = inventory.get_slot(new).expect("We just checked and it existed");
                let size = shape.layout.size();
                let gsize = shape.bounds().size().as_vec2() * style.cell_size;
                let pos = shape.offset.as_vec2() * style.cell_size;
                size_max = size_max.max(pos + gsize);
                size_min = size_min.min(pos);
                let mut offset = shape.offset;
                if offset.x >= 0 {
                    offset.x += 1;
                }
                if offset.y >= 0 {
                    offset.y += 1;
                }
                let mut e = commands.spawn((
                    SlotNode,
                    Node {
                        grid_row: GridPlacement::start_span(offset.y as i16, size.y as u16),
                        grid_column: GridPlacement::start_span(offset.x as i16, size.x as u16),
                        ..Default::default()
                    },
                    ImageNode {
                        image: style.cell_icon.clone(),
                        image_mode: NodeImageMode::Tiled { tile_x: true, tile_y: true, stretch_value: 1.0 },
                        ..Default::default()
                    },
                    ChildOf(rendering),
                    ZIndex(-1),
                    RenderedSlot {
                        inventory: rendering,
                        slot: new.clone(),
                    }
                ));
                if let Some(p) = picicking {
                    e.insert(p.clone());
                }
            }

            for (slot, cell) in update {
                let shape = inventory.get_slot(cell).expect("We just checked and it existed");
                let size = shape.layout.size();
                let gsize = shape.bounds().size().as_vec2() * style.cell_size;
                let pos = shape.offset.as_vec2() * style.cell_size;
                size_max = size_max.max(pos + gsize);
                size_min = size_min.min(pos);
                let mut offset = shape.offset;
                if offset.x >= 0 {
                    offset.x += 1;
                }
                if offset.y >= 0 {
                    offset.y += 1;
                }
                commands.entity(slot).insert((
                Node {
                    grid_row: GridPlacement::start_span(offset.y as i16, size.y as u16),
                    grid_column: GridPlacement::start_span(offset.x as i16, size.x as u16),
                    ..Default::default()
                },
            ));
            }

            // node.width = Val::Px(size_max.x - size_min.x);
            // node.height = Val::Px(size_max.y - size_min.y);

            // TODO update items as well

            // make set of all items in the inventory
            let mut items = inventory.item_entities().collect::<HashSet<_>>();
            if let Some(item_nodes) = item_nodes {
                // iter ItemNodes
                for item in item_nodes.iter() {
                    //get RenderedItem from ItemNode
                    let Ok(target) = render_nodes.get(item) else {
                        warn!("Failed to get RenderedItem component for entity {:?} while updating inventory node for Inventory asset {:?}", item, id);
                        continue;
                    };
                    // remove already existing items from the set
                    if !items.remove(&target.item) {
                        warn!("Despawning ItemNode entity {:?} for removed item in Inventory asset {:?}", item, id);
                        commands.entity(item).despawn();
                    };
                }
            }
            // for all items still in set
            for new in items {
                let shape = inventory.get_shape(new).expect("New items should all definityl be in inventory");
                let size = shape.bounds().size();
                let mut offset = shape.offset;
                if offset.x >= 0 {
                    offset.x += 1;
                } else {
                    offset.x -= 1;
                }
                if offset.y >= 0 {
                    offset.y += 1;
                } else {
                    offset.y -= 1;
                }
                info!("New Offset {}", offset);
                // spawn new ItemNode
                let mut e = commands.spawn((
                    ItemNode(rendering),
                    RenderedItem {
                        item: new,
                    },
                    Node {
                        grid_row: GridPlacement::start_span(offset.y as i16, size.y as u16),
                        grid_column: GridPlacement::start_span(offset.x as i16, size.x as u16),
                        ..Default::default()
                    },
                    ChildOf(rendering),
                ));
                if let Some(p) = picicking {
                    e.insert(p.clone());
                }
            }
        }
    }
}

fn style_inventory_node(
    styles: InventoryStyler,
    changed: Populated<(Entity, &mut Node), Or<(AssetChanged<InventoryStyleHandle>, Changed<InventoryStyleHandle>)>>
) {
    for (entity, mut node) in changed {
        info!("Applying new style to: {}", entity);
        let style = styles.style(entity);
        node.grid_auto_columns = GridTrack::px(style.cell_size.x);
        node.grid_auto_rows = GridTrack::px(style.cell_size.y);
    }
}