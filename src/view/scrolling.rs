use super::{Render, View};

pub struct ScrollingView {
    pub capacity: usize,
    pub index: usize,
    pub skip: usize,
}

impl ScrollingView {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self {
            capacity,
            index: 0,
            skip: 0,
        }
    }
}

impl View for ScrollingView {
    fn render<'a, T>(&mut self, items: &'a [T]) -> Render<&'a T> {
        if items.is_empty() {
            self.index = 0;
            self.skip = 0;
            Render::Empty
        } else {
            if self.skip + self.capacity > items.len() {
                self.skip = items.len().saturating_sub(self.capacity);
            }
            if self.skip + self.index + 1 > items.len() {
                self.index = items.len() - self.skip - 1;
            }
            let skipped = &items[self.skip..];
            let below = skipped[0..self.index].iter().collect();
            let selected = &skipped[self.index];
            let above = (self.index + 1..self.capacity)
                .flat_map(|i| skipped.get(i))
                .collect();
            Render::NonEmpty {
                above,
                selected,
                below,
            }
        }
    }

    fn up(&mut self) {
        if self.index + 1 < self.capacity {
            self.index += 1;
        } else {
            self.skip += 1;
        }
    }

    fn down(&mut self) {
        if self.index > 0 {
            self.index -= 1;
        } else {
            self.skip = self.skip.saturating_sub(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEMS: &[&str] = &[
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
    ];
    const FEW_ITEMS: &[&str] = &["A", "B", "C"];

    #[test]
    fn test_update() {
        // GIVEN
        let mut view = ScrollingView::new(8);

        // WHEN
        let result = view.render(&ITEMS);

        // THEN
        assert_eq!(result.len(), 8);
        assert_eq!(result.num_above(), 7); // 0-indexed
        assert_eq!(result.selected(), Some(&&"A"))
    }

    #[test]
    fn test_up() {
        // GIVEN
        let mut view = ScrollingView::new(8);

        // WHEN
        view.up(); // 6
        view.up(); // 5
        view.up(); // 4
        let result3 = view.render(&ITEMS);

        // THEN
        assert_eq!(result3.len(), 8);
        assert_eq!(result3.num_above(), 4);
    }

    #[test]
    fn test_up_to_extremes() {
        // GIVEN
        let mut view = ScrollingView::new(8);

        // WHEN
        // More than lines_to_show
        view.up();
        view.up();
        view.up();
        view.up();
        view.up();
        view.up();
        view.up();
        view.up();
        view.up();
        view.up();
        view.up();
        view.up();
        view.up();
        let result = view.render(&ITEMS);

        // THEN
        assert_eq!(view.index, 7);
        assert_eq!(view.skip, 5);
        assert_eq!(result.len(), 8);
        assert_eq!(result.num_above(), 0);
    }

    #[test]
    fn test_down_at_bottom() {
        // GIVEN
        let mut view = ScrollingView::new(8);

        // WHEN
        view.down(); // 7
        let result = view.render(&ITEMS);

        // THEN
        assert_eq!(result.len(), 8);
        assert_eq!(result.num_above(), 7);
    }

    #[test]
    fn test_down() {
        // GIVEN
        let mut view = ScrollingView::new(8);

        // WHEN
        view.up(); // 6
        view.up(); // 5
        view.up(); // 4
        view.down(); // 5
        let result = view.render(&ITEMS);

        // THEN
        assert_eq!(result.len(), 8);
        assert_eq!(result.num_above(), 5);
    }

    #[test]
    fn test_few() {
        // GIVEN
        let mut view = ScrollingView::new(8);

        // WHEN
        view.up(); // 6
        view.up(); // 5
        view.up(); // 5
        view.up(); // 5
        let result = view.render(&FEW_ITEMS);

        // THEN
        assert_eq!(result.len(), 3);
        assert_eq!(result.num_above(), 0);
    }
}
