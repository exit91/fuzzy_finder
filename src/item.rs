/// An Item in the list the user sees when searching.

#[derive(Clone)]
pub struct Item<T> {
    /// This is a filler item: there isn't a search result in this place.
    pub name: String,
    pub data: T,
}

impl<T> Item<T> {
    /// Any 'new' item is always non-blank, because it has a name.
    /// Use 'empty' to create a blank item.
    pub fn new(name: String, item: T) -> Self {
        Item::<T> { name, data: item }
    }

    pub fn with_score(self, score: i64, fuzzy_indices: Vec<usize>) -> ScoredItem<T> {
        ScoredItem {
            item: self,
            score,
            fuzzy_indices,
        }
    }
}

#[derive(Clone)]
pub struct ScoredItem<T> {
    pub item: Item<T>,
    pub score: i64,
    pub fuzzy_indices: Vec<usize>,
}
