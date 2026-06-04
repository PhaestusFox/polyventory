use super::*;

#[derive(Component)]
#[require(Node, ImageNode)]
#[relationship(relationship_target = ItemNodes)]
pub struct ItemNode(pub Entity);

#[derive(Component, Default)]
#[relationship_target(relationship = ItemNode)]
pub struct ItemNodes(Vec<Entity>);

pub(super) fn update_item_node_image(
    mut changed: Query<(&RenderedItem, &mut ImageNode, &mut Node, &ItemNode, &mut UiTransform), Changed<RenderedItem>>,
    items: Query<(&Item, &InInventory)>,
    descriptors: Res<Assets<ItemDescriptor>>,
    inventorys: Res<Assets<Inventory>>,
    styles: InventoryStyler,
) {
    for (displayed, mut image, mut node, ItemNode(entity), mut transform) in &mut changed {
        let Ok((item, in_inventory)) = items.get(displayed.item) else {
            warn!("DisplayedItem component references an entity {:?} that does not have an Item component", displayed.item);
            continue;
        };
        let Some(descriptor) = descriptors.get(item.id()) else {
            warn!("Item entity {:?} has an Item component with a handle that does not correspond to an ItemDescriptor asset", displayed.item);
            continue;
        };
        let size = if let Some((image_handle, size)) = descriptor.get_image(&in_inventory.1) {
            image.image = image_handle;
            Some(size)
        } else {
            error!("Item entity {:?} does not have an image compatible with its slot type {:?}", displayed.item, in_inventory.0);
            None
        };
        let Some(inventory) = inventorys.get(in_inventory.0) else {
            warn!("Inventory asset {:?} not found for item entity {:?}", in_inventory.0, displayed.item);
            continue;
        };
        let Some(item) = inventory.get_shape(displayed.item) else {
            warn!("Item entity {:?} not found in inventory {:?}", displayed.item, in_inventory.0);
            continue;
        };
        // let size = size.unwrap_or_else(|| item.bounds().size());
        let size = item.bounds().size();
        let mut offset = item.offset;
        if item.offset.x.is_negative() {
            offset.x -= 1;
        } else {
            offset.x += 1;
        }
        if item.offset.y.is_negative() {
            offset.y -= 1;
        } else {
            offset.y += 1;
        }
        // let size = style.cell_size * size.as_vec2();
        *transform = item.ui_transform();
        node.grid_row = GridPlacement::start_span(offset.y as i16, size.y as u16);
        node.grid_column = GridPlacement::start_span(offset.x as i16, size.x as u16);
    }
} 