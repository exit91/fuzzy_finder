/// Fixed view: only allows selection of the best matches
mod fixed;
/// Scrolling view: allows selection of all matches by scrolling up or down
mod scrolling;

pub use fixed::FixedView;
pub use scrolling::ScrollingView;

pub trait View {
    fn up(&mut self);
    fn down(&mut self);
    fn render<'a, T>(&mut self, items: &'a [T]) -> Render<&'a T>;
}

pub enum Render<T> {
    Empty,
    NonEmpty {
        above: Vec<T>,
        selected: T,
        below: Vec<T>,
    },
}

impl<T> Render<T> {
    pub fn selected(&self) -> Option<&T> {
        match self {
            Render::Empty => None,
            Render::NonEmpty { selected, .. } => Some(selected),
        }
    }

    /// number of items above the selected item
    pub fn num_above(&self) -> usize {
        match self {
            Render::Empty => 0,
            Render::NonEmpty {
                above,
                selected: _,
                below: _,
            } => above.len(),
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Render::Empty => 0,
            Render::NonEmpty {
                above,
                selected: _,
                below,
            } => below.len() + 1 + above.len(),
        }
    }
}

impl<T> IntoIterator for Render<T> {
    type Item = (bool, T);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let vec = match self {
            Render::Empty => vec![],
            Render::NonEmpty {
                above,
                selected,
                below,
            } => {
                let mut vec = Vec::with_capacity(below.len() + 1 + above.len());
                vec.extend(above.into_iter().rev().map(|x| (false, x)));
                vec.push((true, selected));
                vec.extend(below.into_iter().rev().map(|x| (false, x)));
                vec
            }
        };
        vec.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Setup {
        items: Vec<&'static str>,
        few_items: Vec<&'static str>,
        view: ScrollingView,
    }

    impl Setup {
        fn new(lines_to_show: i8) -> Self {
            let view = ScrollingView::new(lines_to_show as usize);

            Setup {
                items: vec![
                    "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
                ],
                few_items: vec!["A", "B", "C"],
                view,
            }
        }
    }

    #[test]
    fn test_update() {
        // GIVEN
        let mut setup = Setup::new(8);

        // WHEN
        let result = setup.view.render(&setup.items);

        // THEN
        assert_eq!(result.len(), 8);
        assert_eq!(result.num_above(), 7); // 0-indexed
        assert_eq!(result.selected(), Some(&&"A"))
    }

    #[test]
    fn test_up() {
        // GIVEN
        let mut setup = Setup::new(8);

        // WHEN
        setup.view.up(); // 6
        setup.view.up(); // 5
        setup.view.up(); // 4
        let result3 = setup.view.render(&setup.items);

        // THEN
        assert_eq!(result3.len(), 8);
        assert_eq!(result3.num_above(), 4);
    }

    #[test]
    fn test_up_to_extremis() {
        // GIVEN
        let mut setup = Setup::new(8);
        let result0 = setup.view.render(&setup.items);
        assert!(setup.items.len() > 0);
        assert_eq!(result0.len(), setup.view.capacity.min(setup.items.len()));

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
        let result = setup.view.render(&setup.items);

        // THEN
        assert_eq!(setup.view.index, 7);
        assert_eq!(setup.view.skip, 5);
        assert_eq!(result.len(), 8);
        assert_eq!(result.num_above(), 0);
    }

    #[test]
    fn test_down_at_bottom() {
        // GIVEN
        let mut setup = Setup::new(8);

        // WHEN
        setup.view.down(); // 7
        let result = setup.view.render(&setup.items);

        // THEN
        assert_eq!(result.len(), 8);
        assert_eq!(result.num_above(), 7);
    }

    #[test]
    fn test_down() {
        // GIVEN
        let mut setup = Setup::new(8);

        // WHEN
        setup.view.up(); // 6
        setup.view.up(); // 5
        setup.view.up(); // 4
        setup.view.down(); // 5
        let result = setup.view.render(&setup.items);

        // THEN
        assert_eq!(result.len(), 8);
        assert_eq!(result.num_above(), 5);
    }

    #[test]
    fn test_few() {
        // GIVEN
        let mut setup = Setup::new(8);

        // WHEN
        setup.view.up(); // 6
        setup.view.up(); // 5
        setup.view.up(); // 5
        setup.view.up(); // 5
        let result = setup.view.render(&setup.few_items);

        // THEN
        assert_eq!(result.len(), 3);
        assert_eq!(result.num_above(), 0);
    }
}
