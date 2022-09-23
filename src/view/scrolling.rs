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
