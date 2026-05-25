use super::*;

#[derive(Component, Default)]
#[require(Node = Node {
    margin: UiRect::all(Val::Auto),
    ..Node::DEFAULT
})]
pub struct InventoryNode;