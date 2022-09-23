use super::{Render, View};

pub struct FixedView {
    pub capacity: usize,
    pub index: usize,
}

impl FixedView {
    pub fn new(capacity: usize) -> Self {
        assert!(capacity > 0);
        Self { capacity, index: 0 }
    }
}
impl View for FixedView {
    fn render<'a, T>(&mut self, items: &'a [T]) -> Render<&'a T> {
        if items.is_empty() {
            self.index = 0;
            Render::Empty
        } else {
            if self.index + 1 > items.len() {
                self.index = items.len() - 1;
            }
            let below = items[0..self.index].iter().collect();
            let selected = &items[self.index];
            let above = items[self.index + 1..self.capacity].iter().collect();
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
        }
    }

    fn down(&mut self) {
        self.index = self.index.saturating_sub(1);
    }
}
