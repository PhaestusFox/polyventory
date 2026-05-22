pub trait Searchable<By> {
    type Index;

    fn contains(&self, filter: By) -> bool;
    fn find(&self, filter: By) -> Option<Self::Index>;
}
