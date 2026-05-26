use bevy::{ecs::{lifecycle::HookContext, world::DeferredWorld}, prelude::*};
use crate::prelude::*;

#[derive(Debug, Component)]
#[component(immutable)]
/// an items sub inventory
pub struct ItemInventory(pub Handle<Inventory>);

#[derive(Debug, Component, Reflect, Clone)]
#[component(immutable, on_remove = Self::remove_from_inventory)]
/// the inventory this item is in
pub struct InInventory(pub AssetId<Inventory>, pub CellType);

impl InInventory {
    fn remove_from_inventory(mut world: DeferredWorld, ctx: HookContext) {
        let Some(InInventory(inventory, _)) = world.get::<Self>(ctx.entity).cloned() else {
            error!("Entity {:?} does not have InInventory but it was removed", ctx.entity);  
            return;
        };
        let mut assets = world.resource_mut::<Assets<Inventory>>();
        let Some(inventory) = assets.get_mut(inventory) else {
            warn!("Inventory {:?} no longer exists", inventory);
            return;
        };
        inventory.remove(ctx.entity);
    }
}