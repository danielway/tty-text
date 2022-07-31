#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Position {
    x: usize,
    y: usize,
}

impl Position {
    pub fn new(x: usize, y: usize) -> Position {
        Position { x, y }
    }

    pub fn x(&self) -> usize {
        self.x
    }

    pub fn y(&self) -> usize {
        self.y
    }

    /// Create a new position from this one with the specified column.
    pub fn set_x(&self, x: usize) -> Position {
        Position { x, y: self.y }
    }

    /// Create a new position from this one with the specified row.
    pub fn set_y(&self, y: usize) -> Position {
        Position { x: self.x, y }
    }

    /// Create a new position from this one with the columns modified as specified.
    pub fn add_x(&self, diff_x: i16) -> Position {
        Position {
            x: (self.x as i16 + diff_x) as usize,
            y: self.y,
        }
    }

    /// Create a new position from this one with the rows modified as specified.
    pub fn add_y(&self, diff_y: i16) -> Position {
        Position {
            x: self.x,
            y: (self.y as i16 + diff_y) as usize,
        }
    }
}

impl Default for Position {
    fn default() -> Self {
        Position::new(0, 0)
    }
}
