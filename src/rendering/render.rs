use std::ops::DerefMut;

use bevy::{
    image::{ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor}, platform::collections::HashMap, sprite::Anchor, ui
};

use super::*;

use crate::{inventory::*, rendering::style::InventoryStyleHandle};

#[derive(Component, Reflect)]
#[relationship(relationship_target = RenderingItem)]
pub struct RenderedItem {
    #[relationship]
    pub item: Entity,
}

#[derive(Component, Default, Reflect)]
#[relationship_target(relationship = RenderedItem)]
pub struct RenderingItem(Vec<Entity>);

