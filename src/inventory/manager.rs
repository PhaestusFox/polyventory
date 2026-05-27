use bevy::platform::collections::HashSet;

use super::*;
use crate::prelude::*;

#[derive(SystemParam)]
pub struct InventoryManager<'w, 's> {
    inventory_assets: ResMut<'w, Assets<Inventory>>,
    commands: Commands<'w, 's>,
    items: Query<'w, 's, (&'static Item, Option<&'static ItemInventory>)>,
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
    items: Query<'w, 's, (&'static Item, Option<&'static ItemInventory>)>,
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
    pub fn add_item(&mut self, item: Entity) -> Result<(), AddFailed> {
        let Ok((handle, sub_inv)) = self.items.get(item) else {
            return Err(AddFailed::EntityIsNotAnItem(item));
        };
        let Some(descriptor) = self.descriptors.get(handle) else {
            return Err(AddFailed::ItemDescriptorNotFound(handle.descriptor.clone()));
        };
        match self.current_inventory.add_item(descriptor, item, sub_inv.map(|s| s.0.id())) {
            Some((slot, shape)) => {
                info!(
                    "Added {} to inventory in slot {:?}@{:?}",
                    descriptor.name(),
                    slot,
                    shape
                );
                self.commands.entity(item).insert(InInventory(self.inv_id, slot));
                Ok(())
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
        shape: Shape,
    ) -> Result<(), AddFailed> {
        let Ok((handle, sub_inv)) = self.items.get(item) else {
            return Err(AddFailed::EntityIsNotAnItem(item));
        };
        let Some(descriptor) = self.descriptors.get(handle) else {
            return Err(AddFailed::ItemDescriptorNotFound(handle.descriptor.clone()));
        };
        for (slot_type, slot_layout) in descriptor.sizes() {
            if shape.layout != *slot_layout {
                continue;
            }
            if self.current_inventory.can_fit(slot_type, &shape) {
                self.current_inventory.insert_item(item, entry::Entry {
                    shape: shape.clone(),
                    sub_inventory: sub_inv.map(|s| s.0.id()),
                });
                return Ok(())
            }
        }
        Err(AddFailed::NoSlotsAcceptThisItem)
    }

    pub fn spawn_item(&mut self, item: Handle<ItemDescriptor>) -> Result<Entity, AddFailed> {
        let Some(descriptor) = self.descriptors.get(&item) else {
            return Err(AddFailed::ItemDescriptorNotFound(item.clone()));
        };
        let Some((cell_type, shape)) = self.fit(descriptor) else {
            return Err(AddFailed::DoesNotFit(self.inv_id));
        };
        self.spawn_item_at_internal(descriptor, item, cell_type, shape)
    }

    pub fn spawn_item_at(&mut self, item: Handle<ItemDescriptor>, offset: IVec2, orientation: Orientation) -> Result<Entity, AddFailed> {
        let Some(descriptor) = self.descriptors.get(&item) else {
            return Err(AddFailed::ItemDescriptorNotFound(item.clone()));
        };
        let mut shape = Shape {
            layout: Layout::default(),
            offset,
            orientation,
        };
        for (cell_type, slot_layout) in descriptor.sizes() {
            shape.layout = slot_layout.clone();
            if !self.current_inventory.can_fit(cell_type, &shape) {
                continue;
            }
            return self.spawn_item_at_internal(descriptor, item, cell_type.clone(), shape);
        }
        Err(AddFailed::DoesNotFit(self.inv_id))
    }

    fn spawn_item_at_internal(&mut self, descriptor: &ItemDescriptor, item: Handle<ItemDescriptor>, cell_type: CellType, shape: Shape) -> Result<Entity, AddFailed> {
        // todo - clean up if failed to add to inventory
        let entity = self
            .commands
            .spawn((Item::new(item), descriptor.spawn(), InInventory(self.inv_id, cell_type))).id();
        let sub: Option<AssetId<Inventory>>;
        if let Some(sub_inventory) = descriptor.sub_inventory() {
            let Some(inv_des) = self.inventory_descriptors.get(sub_inventory) else {
                return Err(AddFailed::InventoryDescriptorNotFound(sub_inventory.clone()));
            };
            let mut inv = inv_des.create_inventory();
            inv.name = format!("{}({:?}) Inventory", descriptor.name(), entity);
            let inv_h = self.all_inventories.add(inv);
            sub = Some(inv_h.id());
            self.commands.entity(entity).insert(ItemInventory(inv_h));
        } else {
            sub = None;
        };

        self.insert_item(entity, entry::Entry {
            shape,
            sub_inventory: sub,
        });
        Ok(entity)
    }

    /// Remove an item from the inventory recursively searching through any sub-inventories.
    /// Returns the inventory the item was removed from
    pub fn remove_item(&mut self, item: Entity) -> Result<AssetId<Inventory>, RemoveFailed> {
        let mut checked = HashSet::new();
        match self.current_inventory.find(item, &self.all_inventories, &mut checked) {
            Some(FoundItem::InSelf) => {
                self.current_inventory.remove(item).ok_or(RemoveFailed::FailedToRemove)?;
                Ok(self.inv_id)
            },
            Some(FoundItem::InSubInventory(inv)) => {
                let Some(inventory) = self.all_inventories.get_mut(inv) else {
                    error!("Failed to get inventory {:?} while trying to remove item {:?}. This should not happen since we just found the item in this inventory. Skipping.", inv, item);
                    return Err(RemoveFailed::InventoryNotFound(inv));
                };
                inventory.remove(item).ok_or(RemoveFailed::FailedToRemove)?;
                Ok(inv)
            },
            None => Err(RemoveFailed::ItemNotFound(item)),
        }
    }

    fn find(check: AssetId<Inventory>, item: Entity, assets: &Assets<Inventory>, checked: &mut HashSet<AssetId<Inventory>>) -> Option<AssetId<Inventory>> {
        let Some(inventory) = assets.get(check) else {
            warn!("Failed to get inventory {:?} while trying to find item {:?}. Skipping.", check, item);
            return None;
        };
        if inventory.contains(item) {
            return Some(check);
        }
        checked.insert(check);
        for entry in inventory.iter_sub_inventories() {
            if checked.contains(&entry) {
                error!("Inventory {:?} has a circular reference. Already checked this inventory while looking for item {:?}. Skipping.", entry, item);
                continue;
            }
            if let Some(found) = Self::find(entry, item, assets, checked) {
                return Some(found);
            }
        }
        None
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
    #[error("There is not space big enough to put this it in the inventory")]
    DoesNotFit(AssetId<Inventory>),
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveFailed {
    #[error("Failed to remove item from inventory: entity {0} not found in any slot")]
    ItemNotFound(Entity),
    #[error("Failed to remove item from inventory")]
    FailedToRemove,
    #[error("Failed to remove item from inventory: inventory asset {0:?} not found")]
    InventoryNotFound(AssetId<Inventory>),
}
