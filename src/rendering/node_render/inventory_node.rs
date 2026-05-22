use super::*;

#[derive(Component)]
#[require(Node = Node {
    margin: UiRect::all(Val::Auto),
    ..Node::DEFAULT
})]
pub struct InventoryNode(pub Handle<Inventory>);

impl Into<AssetId<Inventory>> for &InventoryNode {
    fn into(self) -> AssetId<Inventory> {
        self.0.id()
    }
}

impl InventoryNode {
    pub fn new(inventory: Handle<Inventory>) -> Self {
        Self(inventory)
    }
}