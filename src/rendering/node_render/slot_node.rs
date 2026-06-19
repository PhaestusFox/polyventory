use super::*;

#[derive(Component)]
#[require(Node)]
pub struct SlotNode;

pub fn update_image_cell_scale(
    mut images: Populated<(&mut ImageNode, &RenderedSlot), Added<SlotNode>>,
    styles: InventoryStyler,
    image_assets: Res<Assets<Image>>,
) {
    for (mut image, slot) in &mut images {
        let style = styles.style(slot.inventory);
        let Some(image_asset) = image_assets.get(&image.image) else {
            warn!(
                "Failed to get Image asset for image handle {:?}",
                image.image
            );
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
