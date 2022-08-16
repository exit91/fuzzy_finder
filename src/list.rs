/// The list and events for handling movement within the list. No UI.
pub struct List<T>
where
    T: Clone,
{
    /// maximum number of items
    ///
    /// it must satisfy:
    /// capacity > 0
    capacity: usize,
    /// vec of items, ordered from bottom to top
    items: Vec<T>,
    /// index of the selected item
    ///
    /// it satisfies:
    /// index < items.len() || items.is_empty() && index == 0
    index: usize,
}

impl<T> List<T>
where
    T: Clone,
{
    /// Initialize a `List` with some `capacity` > 0
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        List {
            capacity,
            items: Vec::with_capacity(capacity),
            index: 0,
        }
    }

    /// Items in order from top to bottom
    pub fn items<'a>(&'a self) -> Box<dyn Iterator<Item = T> + 'a> {
        Box::new(self.items.iter().rev().cloned())
    }

    /// Items in order from top to bottom
    pub fn tagged_iter<'a>(&'a self) -> Box<dyn Iterator<Item = (bool, T)> + 'a> {
        let index_from_top = self.len() - self.index;
        Box::new(
            self.items()
                .enumerate()
                .map(move |(index, item)| (index == index_from_top, item)),
        )
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// number of items above the selected one
    #[allow(dead_code)]
    pub fn num_above(&self) -> Option<usize> {
        self.items.len().checked_sub(1).map(|max| max - self.index)
    }

    pub fn up(&mut self) {
        if !self.items.is_empty() {
            self.index = (self.index + 1).min(self.items.len() - 1);
        }
    }

    pub fn down(&mut self) {
        if !self.items.is_empty() {
            self.index = self.index.saturating_sub(1);
        }
    }

    /// Takes the current matches and updates the visible contents.
    ///
    /// The input matches are assumed to be sorted in descending order of score.
    pub fn update(&mut self, matches: &[T]) {
        log::info!("Updating view with {} match(es)", matches.len());

        self.items.clear();
        self.items
            .extend(matches.iter().take(self.capacity).cloned());

        // ensure valid index
        if let Some(max_index) = self.items.len().checked_sub(1) {
            self.index = self.index.min(max_index);
        } else {
            // the list is empty
            self.index = 0;
        }
    }

    pub fn get_selected(&self) -> &T {
        self.items.get(self.index).unwrap()
    }
}

#[cfg(test)]
mod tests {

    use super::super::item::Item;
    use super::*;

    #[derive(Clone)]
    struct TestItem {
        name: String,
    }

    fn item(name: &str) -> Item<TestItem> {
        Item::new(
            String::from(name),
            TestItem {
                name: String::from(name),
            },
        )
    }

    struct Setup {
        items: Vec<Item<TestItem>>,
        few_items: Vec<Item<TestItem>>,
        view: List<Item<TestItem>>,
    }

    impl Setup {
        fn new(lines_to_show: i8) -> Self {
            let view = List::<Item<TestItem>>::new(lines_to_show as usize);

            Setup {
                items: vec![
                    item("A"),
                    item("B"),
                    item("C"),
                    item("D"),
                    item("E"),
                    item("F"),
                    item("G"),
                    item("H"),
                    item("I"),
                    item("J"),
                    item("K"),
                    item("L"),
                    item("M"),
                ],
                few_items: vec![item("A"), item("B"), item("C")],
                view,
            }
        }
    }

    #[test]
    fn test_update() {
        // GIVEN
        let mut setup = Setup::new(8);

        // WHEN
        setup.view.update(&setup.items);

        // THEN
        assert_eq!(setup.view.len(), 8);
        assert_eq!(setup.view.num_above(), Some(7)); // 0-indexed
        assert_eq!(setup.view.get_selected().item.as_ref().unwrap().name, "A")
    }

    #[test]
    fn test_up() {
        // GIVEN
        let mut setup = Setup::new(8);
        setup.view.update(&setup.items);

        // WHEN
        setup.view.up(); // 6
        setup.view.up(); // 5
        setup.view.up(); // 4

        // THEN
        assert_eq!(setup.view.len(), 8);
        assert_eq!(setup.view.num_above(), Some(4));
    }

    #[test]
    fn test_up_to_extremis() {
        // GIVEN
        let mut setup = Setup::new(8);
        setup.view.update(&setup.items);
        assert!(setup.items.len() > 0);
        assert_eq!(
            setup.view.len(),
            setup.view.capacity().min(setup.items.len())
        );

        // WHEN
        // More than lines_to_show
        setup.view.up();
        setup.view.up();
        setup.view.up();
        setup.view.up();
        setup.view.up();
        setup.view.up();
        setup.view.up();
        setup.view.up();
        setup.view.up();
        setup.view.up();
        setup.view.up();
        setup.view.up();
        setup.view.up();

        // THEN
        assert_eq!(setup.view.len(), 8);
        assert_eq!(setup.view.num_above(), Some(0));
    }

    #[test]
    fn test_down_at_bottom() {
        // GIVEN
        let mut setup = Setup::new(8);
        setup.view.update(&setup.items);

        // WHEN
        setup.view.down(); // 7

        // THEN
        assert_eq!(setup.view.len(), 8);
        assert_eq!(setup.view.num_above(), Some(7));
    }

    #[test]
    fn test_down() {
        // GIVEN
        let mut setup = Setup::new(8);
        setup.view.update(&setup.items);

        // WHEN
        setup.view.up(); // 6
        setup.view.up(); // 5
        setup.view.up(); // 4
        setup.view.down(); // 5

        // THEN
        assert_eq!(setup.view.len(), 8);
        assert_eq!(setup.view.num_above(), Some(5));
    }

    #[test]
    fn test_few() {
        // GIVEN
        let mut setup = Setup::new(8);

        // WHEN
        setup.view.update(&setup.few_items);
        setup.view.up(); // 6
        setup.view.up(); // 5
        setup.view.up(); // 5
        setup.view.up(); // 5

        // THEN
        assert_eq!(setup.view.len(), 3);
        assert_eq!(setup.view.num_above(), Some(0));
    }
}
