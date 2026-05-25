use bevy::ecs::system::{SystemParam};
use bevy::prelude::*;
use crate::prelude::*;
use super::*;

pub struct MouseInventoryPlugin;

impl Plugin for MouseInventoryPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(detect_pickup);
    }
}

fn detect_pickup(
    click: On<Pointer<Click>>,
    icons: Query<&RenderedItem>,
    mut commands: Commands,
) {
    let Ok(rendered_item) = icons.get(click.entity) else {
        return;
    };
    println!("Clicked on item entity {:?} with item {:?}", click.entity, rendered_item.item);
    commands.trigger(PickupItem(rendered_item.item));
}