use super::*;

#[derive(SystemParam)]
pub struct InventoryManager<'w, 's> {
    inventory_assets: ResMut<'w, Assets<Inventory>>,
    commands: Commands<'w, 's>,
    items: Query<'w, 's, &'static Item>,
    descriptors: Res<'w, Assets<ItemDescriptor>>,
    inventory_descriptors: Res<'w, Assets<InventoryDescriptor>>,
}

impl InventoryManager<'_, '_> {
    pub fn open_inventory(
        &mut self,
        inventory: impl Into<AssetId<Inventory>>,
    ) -> Option<InventoryCommands<'_, '_, '_>> {
        let id = inventory.into();
        let current = self.inventory_assets.remove_untracked(id)?;
        Some(InventoryCommands {
            commands: self.commands.reborrow(),
            current_inventory: current,
            inv_id: id,
            items: self.items.reborrow(),
            descriptors: &self.descriptors,
            inventory_descriptors: &self.inventory_descriptors,
            all_inventories: &mut self.inventory_assets,
        })
    }

    pub fn create_inventory(&mut self, inventory: Inventory) -> Handle<Inventory> {
        self.inventory_assets.add(inventory)
    }

    pub fn find_item(&self, item: Entity) -> Option<AssetId<Inventory>> {
        for (handle, inventory) in self.inventory_assets.iter() {
            if inventory.contains(item) {
                return Some(handle);
            }
        }
        None
    }

    pub fn get_strong(&mut self, id: AssetId<Inventory>) -> Option<Handle<Inventory>> {
        self.inventory_assets.get_strong_handle(id)
    }
}

pub struct InventoryCommands<'w, 's, 'a> {
    pub commands: Commands<'w, 's>,
    current_inventory: Inventory,
    inv_id: AssetId<Inventory>,
    items: Query<'w, 's, &'static Item>,
    descriptors: &'a Assets<ItemDescriptor>,
    inventory_descriptors: &'a Assets<InventoryDescriptor>,
    all_inventories: &'a mut Assets<Inventory>,
}

impl Drop for InventoryCommands<'_, '_, '_> {
    fn drop(&mut self) {
        let i = core::mem::take(&mut self.current_inventory);
        _ = self.all_inventories.insert(self.inv_id, i).unwrap();
    }
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
            Some((slot, shape, slot_type)) => {
                info!(
                    "Added {} to inventory in slot {}@{} with slot type {:?}",
                    descriptor.name(),
                    slot,
                    shape.position,
                    slot_type
                );
                self.commands.entity(item).insert((shape, slot_type));
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
        // self.all_inventories.get_handle_provider().
        // todo - clean up if failed to add to inventory
        let entity = self
            .commands
            .spawn((Item::new(item), descriptor.spawn(), InInventory(self.inv_id))).id();
        if let Some(sub_inventory) = descriptor.sub_inventory() {
            let Some(inv_des) = self.inventory_descriptors.get(sub_inventory) else {
                return Err(AddFailed::InventoryDescriptorNotFound(sub_inventory.clone()));
            };
            let mut inv = inv_des.create_inventory();
            inv.name = format!("{}({:?}) Inventory", descriptor.name(), entity);
            let inv_h = self.all_inventories.add(inv);
            self.commands.entity(entity).insert(ItemInventory(inv_h));
            warn!("Spawn sub inventory for {}", descriptor.name());
        }

        match self.current_inventory.add_item(descriptor, entity) {
            Some((slot, shape, slot_type)) => {
                info!(
                    "Added {} to inventory in slot {}@{} with slot type {:?}",
                    descriptor.name(),
                    slot,
                    shape.position,
                    slot_type
                );
                self.commands.entity(entity).insert((shape, slot_type));
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
        let (slot, shape, slot_type) = self
            .current_inventory
            .reserve_item_at(descriptor, position, orientation)?;
        let entity = self
            .commands
            .spawn((Item::new(item), descriptor.spawn(), slot_type, shape.clone()))
            .id();
        self.current_inventory
            .add_unchecked(slot, shape, entity)
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
        &self.current_inventory
    }
}

impl core::ops::DerefMut for InventoryCommands<'_, '_, '_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.current_inventory
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AddFailed {
    #[error("Entity({0}) does not have an Item component")]
    EntityIsNotAnItem(Entity),
    #[error("ItemDescriptor {0:?} not found")]
    ItemDescriptorNotFound(Handle<ItemDescriptor>),
    #[error("InventoryDescriptor {0:?} not found")]
    InventoryDescriptorNotFound(Handle<InventoryDescriptor>),
    #[error("Failed to add item to inventory")]
    NotYetFullImplemented,
    #[error("This inventory does not have enough slots to fit: {} < {}", num_slots, slot_index + 1)]
    SlotNotInInventory { slot_index: usize, num_slots: usize },
    #[error("No slots in inventory")]
    NoSlotsInInventory,
    #[error("Item bounds {item_bounds:?} do not fit within slot {slot_index} bounds {slot_bounds:?}")]
    NotInBounds { slot_index: usize, item_bounds: Aabb2d, slot_bounds: Aabb2d },
    #[error("Item Overlaps with existing item in slot {slot_index} at entry {overlap_index}")]
    OverlapWithExistingItem { slot_index: usize, overlap_index: usize},
    #[error("No slots in inventory accept this item")]
    NoSlotsAcceptThisItem,
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveFailed {
    #[error("Failed to remove item from inventory: entity {0} not found in any slot")]
    ItemNotFound(Entity),
    #[error("Failed to remove item from inventory")]
    FailedToRemove,
}
