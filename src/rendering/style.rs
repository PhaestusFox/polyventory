use bevy::ecs::system::SystemParam;

use super::*;

#[derive(Asset, TypePath)]
pub struct InventoryStyle {
    pub cell_size: Vec2,
    pub cell_icon: Handle<Image>,
    pub background: Option<Handle<Image>>,
}

pub struct InventoryStyleAsset {
    pub cell_size: Vec2,
    pub cell_icon: Option<String>,
    pub background: Option<String>,
}

#[derive(Component, Deref)]
pub struct InventoryStyleHandle(pub Handle<InventoryStyle>);


impl Into<AssetId<InventoryStyle>> for &InventoryStyleHandle {
    fn into(self) -> AssetId<InventoryStyle> {
        self.0.id()
    }
}

pub const DEFAULT_CELL_SIZE: Vec2 = Vec2::new(32.0, 32.0);
pub const DEFAULT_INVENTORY_STYLE: InventoryStyleAsset = InventoryStyleAsset {
    cell_size: DEFAULT_CELL_SIZE,
    cell_icon: None,
    background: None,
};

pub(crate) fn register_default_style(app: &mut App, default_style: Option<&InventoryStyleAsset>) {
    let style = default_style.unwrap_or(&DEFAULT_INVENTORY_STYLE);
    let asset_server = app.world_mut().resource::<AssetServer>().clone();
    let style = InventoryStyle {
        cell_size: style.cell_size,
        cell_icon: style.cell_icon.as_ref().map(|path| asset_server.load(path)).unwrap_or(InventoryStyler::FALLBACK_STYLE.cell_icon),
        background: style
            .background
            .as_ref()
            .map(|path| asset_server.load(path)),
    };
    let mut assets = app.world_mut().resource_mut::<Assets<InventoryStyle>>();
    assets.insert(AssetId::default(), style).unwrap();
}


#[derive(SystemParam)]
pub struct InventoryStyler<'w, 's> {
    styles: Res<'w, Assets<InventoryStyle>>,
    inventorys: Query<'w, 's, &'static InventoryStyleHandle>,
}

impl InventoryStyler<'_, '_> {
    const FALLBACK_STYLE: InventoryStyle = InventoryStyle {
        cell_size: DEFAULT_CELL_SIZE,
        cell_icon: Handle::Uuid(AssetId::<Image>::DEFAULT_UUID, core::marker::PhantomData),
        background: None,
    };
    pub fn style(&self, entity: Entity) -> &InventoryStyle {
        match self.inventorys.get(entity) {
            Err(_) => self.get_default(),
            Ok(handle) => self.styles.get(handle).unwrap_or_else(|| {
                warn!("Style handle {:?} not found, falling back to default", handle.0);
                self.get_default()
            })
        }
    }

    pub fn get_default(&self) -> &InventoryStyle {
        self.styles.get(AssetId::default()).unwrap_or(&Self::FALLBACK_STYLE)
    }
}