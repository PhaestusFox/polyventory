use std::ops::DerefMut;

use bevy::{
    image::{ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor}, platform::collections::HashMap, sprite::Anchor, ui
};

use super::*;

use crate::{inventory::*, rendering::style::InventoryStyleHandle};

#[derive(Event)]
pub enum SpawnInventory {
    Sprite(Handle<Inventory>),
    Ui(Handle<Inventory>),
}

pub(super) fn spawn_inventory_window(
    on: On<SpawnInventory>,
    mut commands: Commands,
    inventorys: Res<Assets<Inventory>>,
    styles: Res<Assets<InventoryStyle>>,
    items: Query<&Item>,
    descriptors: Res<Assets<ItemDescriptor>>,
) {
    let target= match on.event() {
        SpawnInventory::Sprite(handle) => handle,
        SpawnInventory::Ui(handle) => {
            commands.spawn((
                Name::new("InventoryNode"),
                super::node_render::InventoryNode,
                RenderedInventory::new(handle.clone()),
            ));
            return;
        },
    };
    let Some(inventory) = inventorys.get(target.id()) else {
        error!("No Inventory asset found for handle");
        return;
    };
    let style = styles
        .get(AssetId::default())
        .expect("Default Style must be loaded");
    let root = commands
        .spawn((
            Name::new("Inventory Window"),
            Transform::default(),
            Visibility::default(),
            InventorySprite,
            RenderedInventory::new(target.clone()),
        ))
        .id();
    for (index, slot) in inventory.slots().iter().enumerate() {
        commands.spawn((
                Name::new(format!("Slot {}", index)),
                Sprite {
                    custom_size: Some(style.cell_size * slot.size.as_vec2()),
                    image: style.cell_icon.clone(),
                    image_mode: SpriteImageMode::Tiled { tile_x: true, tile_y: true, stretch_value: 1. },
                    ..Default::default()
                },
                Anchor::BOTTOM_LEFT,
                Transform::from_translation((style.cell_size * slot.position.as_vec2()).extend(0.0)),
                Pickable::default(),
                RenderedSlot {
                    inventory: root,
                    index,
                },
                ChildOf(root),
            )).with_children(|slot_root| {
                for item in &slot.entries {
                    let Ok(item_entity) = items.get(item.entity) else {
                        warn!("Item entity {:?} does not have an Item component", item.entity);
                        continue;
                    };
                    let Some(descriptor) = descriptors.get(item_entity.id()) else {
                        warn!("Item entity {:?} has an Item component with a handle that does not correspond to an ItemDescriptor asset", item.entity);
                        continue;
                    };
                    let (image, size) = match descriptor.image(slot.slot_type.iter().cloned()) {
                        Some(image) => image,
                        None => {
                            error!("Item entity {:?} does not have an image compatible with any of this slots types {:?}", item.entity, slot.slot_type);
                            continue;
                        },
                    };
                    slot_root.spawn((
                        Name::new(format!("Item {:?}", descriptor.name())),
                        Transform::IDENTITY,
                        Visibility::Visible,
                        Sprite {
                            custom_size: Some(style.cell_size * size.as_vec2()),
                            image,
                            ..Default::default()
                        },
                        Anchor::BOTTOM_LEFT,
                        RenderedItem { item: item.entity},
                        item.shape.clone(),
                        Pickable::default(),
                    ));
                }
            });
    }
}

#[derive(Component, Reflect)]
#[relationship(relationship_target = RenderingItem)]
pub struct RenderedItem {
    #[relationship]
    pub item: Entity,
}

#[derive(Component, Default)]
#[relationship_target(relationship = RenderedItem)]
pub struct RenderingItem(Vec<Entity>);

pub(super) fn update_displayed_item_transform(
    mut changed: Query<(&Shape, &mut Transform, Option<&ChildOf>), Changed<Shape>>,
    slots: Query<&RenderedSlot>,
    inventorys: Query<&InventoryStyleHandle>,
    styles: Res<Assets<InventoryStyle>>,
) {
    for (shape, mut transform, parent) in &mut changed {
        let size = if let Some(slot) = parent {
            if let Ok(slot) = slots.get(slot.parent())
                && let Ok(style) = inventorys.get(slot.inventory)
            {
                styles
                    .get(style.id())
                    .or_else(|| styles.get(AssetId::default()))
                    .map(|v| v.cell_size)
                    .unwrap_or(super::style::DEFAULT_CELL_SIZE)
            } else {
                styles
                    .get(AssetId::default())
                    .map(|v| v.cell_size)
                    .unwrap_or(super::style::DEFAULT_CELL_SIZE)
            }
        } else {
            warn!("No parent");
            Vec2::ZERO
        };
        let mut new_transform = shape.transform(size);
        new_transform.translation.z = 1.;
        *transform = new_transform;
    }
}

fn something(
    item: &[Entity]
) {
}
