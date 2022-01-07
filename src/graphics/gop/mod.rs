mod buffer;
pub mod writer;

pub use writer::{Direction, TextWriter};

#[derive(Clone, Copy)]
pub struct Position {
    x: usize,
    y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }

    pub fn flat(&self) -> usize {
        self.x * self.y
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(0, 0)
    }
}
