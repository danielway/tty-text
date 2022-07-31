#[derive(Debug)]
pub struct LineLayout {
    rows: Vec<RowLayout>,
}

impl LineLayout {
    pub fn new(rows: Vec<RowLayout>) -> Self {
        Self { rows }
    }

    pub fn rows(&self) -> &Vec<RowLayout> {
        &self.rows
    }
}

#[derive(Debug)]
pub struct RowLayout {
    widths: Vec<usize>,
}

impl RowLayout {
    pub fn new(widths: Vec<usize>) -> Self {
        Self { widths }
    }

    pub fn widths(&self) -> &Vec<usize> {
        &self.widths
    }
}
