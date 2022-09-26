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
