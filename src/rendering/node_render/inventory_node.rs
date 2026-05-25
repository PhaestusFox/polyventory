use super::*;

#[derive(Component)]
#[require(Node = Node {
    margin: UiRect::all(Val::Auto),
    ..Node::DEFAULT
})]
pub struct InventoryNode;