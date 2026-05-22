use super::*;

#[derive(SystemParam)]
pub struct InventoryManager<'w, 's> {
    inventory_assets: ResMut<'w, Assets<Inventory>>,
    commands: Commands<'w, 's>,
    items: Query<'w, 's, &'static Item>,
    descriptors: Res<'w, Assets<ItemDescriptor>>,
}

impl InventoryManager<'_, '_> {
    pub fn open_inventory(
        &mut self,
        inventory: impl Into<AssetId<Inventory>>,
    ) -> Option<InventoryCommands<'_, '_, '_>> {
        let current = self.inventory_assets.get_mut(inventory)?;

        Some(InventoryCommands {
            commands: self.commands.reborrow(),
            current_inventory: current,
            items: self.items.reborrow(),
            descriptors: &self.descriptors,
        })
    }

    pub fn create_inventory(&mut self, inventory: Inventory) -> Handle<Inventory> {
        self.inventory_assets.add(inventory)
    }
}

pub struct InventoryCommands<'w, 's, 'a> {
    commands: Commands<'w, 's>,
    current_inventory: &'a mut Inventory,
    items: Query<'w, 's, &'static Item>,
    descriptors: &'a Assets<ItemDescriptor>,
}

impl InventoryCommands<'_, '_, '_> {
    /// Add an existing item from the world to this inventory.
    pub fn add_item(&mut self, item: Entity) -> Result<usize, AddFailed> {
        let Ok(handle) = self.items.get(item) else {
            return Err(AddFailed::EntityIsNotAnItem(item));
        };
        let Some(descriptor) = self.descriptors.get(handle) else {
            return Err(AddFailed::ItemDescriptorNotFound(handle.descriptor.clone()));
        };
        match self.current_inventory.add_item(descriptor, item) {
            Some((slot, shape)) => {
                info!(
                    "Added {} to inventory in slot {}@{}",
                    descriptor.name(),
                    slot,
                    shape.position
                );
                self.commands.entity(item).insert((shape,));
                Ok(slot)
            }
            None => {
                warn!(
                    "Failed to add {} to inventory: not yet fully implemented",
                    descriptor.name()
                );
                Err(AddFailed::NotYetFullImplemented)
            }
        }
    }

    pub fn add_item_at(
        &mut self,
        item: Entity,
        position: IVec2,
        orientation: Orientation,
    ) -> Result<(usize, Shape), AddFailed> {
        let Ok(handle) = self.items.get(item) else {
            return Err(AddFailed::EntityIsNotAnItem(item));
        };
        let Some(descriptor) = self.descriptors.get(handle) else {
            return Err(AddFailed::ItemDescriptorNotFound(handle.descriptor.clone()));
        };
        match self
            .current_inventory
            .add_item_at(descriptor, item, position, orientation)
        {
            Ok((slot, shape)) => {
                info!(
                    "Added {} to inventory in slot {}@{:?} with orientation {:?}",
                    descriptor.name(),
                    slot,
                    shape.position,
                    shape.orientation
                );
                self.commands.entity(item).insert((shape.clone(),));
                Ok((slot, shape))
            }
            Err(e) => {
                warn!(
                    "Failed to add {} to inventory: {:?} (not yet fully implemented)",
                    descriptor.name(),
                    e
                );
                Err(AddFailed::NotYetFullImplemented)
            }
        }
    }

    pub fn spawn_item(&mut self, item: Handle<ItemDescriptor>) -> Result<Entity, AddFailed> {
        let Some(descriptor) = self.descriptors.get(&item) else {
            return Err(AddFailed::ItemDescriptorNotFound(item.clone()));
        };
        let entity = self
            .commands
            .spawn((Item::new(item), descriptor.spawn()))
            .id();
        match self.current_inventory.add_item(descriptor, entity) {
            Some((slot, shape)) => {
                info!(
                    "Added {} to inventory in slot {}@{}",
                    descriptor.name(),
                    slot,
                    shape.position
                );
                self.commands.entity(entity).insert((shape,));
                Ok(entity)
            }
            None => {
                warn!(
                    "Failed to add {} to inventory: not yet fully implemented",
                    descriptor.name()
                );
                Err(AddFailed::NotYetFullImplemented)
            }
        }
    }

    pub fn spawn_item_at(
        &mut self,
        item: Handle<ItemDescriptor>,
        position: IVec2,
        orientation: Orientation,
    ) -> Result<Entity, AddFailed> {
        let Some(descriptor) = self.descriptors.get(&item) else {
            return Err(AddFailed::ItemDescriptorNotFound(item.clone()));
        };
        let (slot, shape) = self
            .current_inventory
            .reserve_item_at(descriptor, position, orientation)
            .map_err(|_| AddFailed::NotYetFullImplemented)?;
        let entity = self
            .commands
            .spawn((Item::new(item), descriptor.spawn()))
            .id();
        self.current_inventory
            .add_unchecked(slot, shape.clone(), entity)
            .map_err(|_| AddFailed::SlotNotInInventory {
                slot_index: slot,
                num_slots: self.current_inventory.slots().len(),
            })?;
        Ok(entity)
    }

    pub fn remove_item(&mut self, item: Entity) -> Result<(usize, Entry), RemoveFailed> {
        use crate::inventory::traits::Searchable;
        for (i, slot) in self.current_inventory.iter_slots_mut().enumerate() {
            let Some(index) = slot.find(item) else {
                continue;
            };
            return slot
                .remove(index)
                .map(|entry| (i, entry))
                .ok_or(RemoveFailed::FailedToRemove);
        }
        Err(RemoveFailed::ItemNotFound(item))
    }
}

impl core::ops::Deref for InventoryCommands<'_, '_, '_> {
    type Target = Inventory;

    fn deref(&self) -> &Self::Target {
        self.current_inventory
    }
}

impl core::ops::DerefMut for InventoryCommands<'_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.current_inventory
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AddFailed {
    #[error("Entity({0}) does not have an Item component")]
    EntityIsNotAnItem(Entity),
    #[error("ItemDescriptor {0:?} not found")]
    ItemDescriptorNotFound(Handle<ItemDescriptor>),
    #[error("Failed to add item to inventory")]
    NotYetFullImplemented,
    #[error("This inventory does not have enough slots to fit: {} < {}", num_slots, slot_index + 1)]
    SlotNotInInventory { slot_index: usize, num_slots: usize },
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveFailed {
    #[error("Failed to remove item from inventory: entity {0} not found in any slot")]
    ItemNotFound(Entity),
    #[error("Failed to remove item from inventory")]
    FailedToRemove,
}
