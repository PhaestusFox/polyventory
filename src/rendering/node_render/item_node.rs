use super::*;

#[derive(Component)]
#[require(Node, ImageNode)]
pub struct ItemNode(pub Entity);

pub(super) fn update_item_node_image(
    mut changed: Query<(&RenderedItem, &mut ImageNode, &mut Node, &ItemNode, &mut UiTransform), Changed<RenderedItem>>,
    items: Query<(&Item, &Shape, &SlotType)>,
    descriptors: Res<Assets<ItemDescriptor>>,
    styles: InventoryStyler,
) {
    for (displayed, mut image, mut node, ItemNode(entity), mut transform) in &mut changed {
        let Ok((item, shape, slot_type)) = items.get(displayed.item) else {
            warn!("DisplayedItem component references an entity {:?} that does not have an Item component", displayed.item);
            continue;
        };
        let Some(descriptor) = descriptors.get(item.id()) else {
            warn!("Item entity {:?} has an Item component with a handle that does not correspond to an ItemDescriptor asset", displayed.item);
            continue;
        };
        let Some((image_handle, size)) = descriptor.get_image(slot_type) else {
            error!("Item entity {:?} does not have an image compatible with its slot type {:?}", displayed.item, slot_type);
            continue;
        };
        let style = styles.style(*entity);
        let size = style.cell_size * size.as_vec2();
        *transform = shape.ui_transform(style.cell_size);
        node.position_type = PositionType::Absolute;
        node.width = Val::Px(size.x);
        node.height = Val::Px(size.y);
        image.image = image_handle;
    }
} 