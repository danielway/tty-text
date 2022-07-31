use crate::key::Key;
use crate::layout::{LineLayout, RowLayout};
use crate::position::Position;
use unicode_segmentation::UnicodeSegmentation;

pub struct Text {
    cursor: Position,
    lines: Vec<String>,
    layout: Vec<LineLayout>,
}

impl Text {
    pub fn new() -> Text {
        Text {
            cursor: Position::default(),
            lines: vec![String::new()],
            layout: Vec::new(),
        }
    }

    pub fn value(&self) -> String {
        self.lines.join("")
    }

    pub fn lines(&self) -> &Vec<String> {
        &self.lines
    }

    pub fn cursor(&self) -> Position {
        self.cursor
    }

    pub fn set_layout(&mut self, layout: Vec<LineLayout>) {
        self.layout = layout;
    }

    pub fn update(&mut self, key: Key) {
        match key {
            Key::Char(ch) => {
                let (line_index, grapheme_index) = self.get_cursor_line_and_grapheme_indexes();
                let line = &self.lines[line_index];
                let (byte_index, _) = self.get_grapheme_byte_information(line, grapheme_index);

                let before = line.graphemes(true).count();

                let line = &mut self.lines[line_index];
                line.insert(byte_index, ch);

                let after = line.graphemes(true).count();

                if after > before {
                    self.cursor = self.cursor.add_x(1);
                }
            }
            Key::Left => {
                if self.cursor.x() > 0 {
                    self.cursor = self.cursor.add_x(-1);
                } else {
                    if self.cursor.y() > 0 {
                        let prior_row_length = self.get_row_length(self.cursor.y() - 1);
                        self.cursor = self.cursor.add_y(-1).set_x(prior_row_length);
                    }
                }
            }
            Key::Right => {
                let row_length = self.get_row_length(self.cursor.y());
                if self.cursor.x() < row_length {
                    self.cursor = self.cursor.add_x(1);
                } else {
                    if self.cursor.y() + 1 < self.get_total_row_count() {
                        self.cursor = self.cursor.add_y(1).set_x(0);
                    }
                }
            }
            Key::Up => {
                if self.cursor.y() > 0 {
                    let cursor_column = self.get_cursor_column();
                    let previous_row = self.get_row(self.cursor.y() - 1);

                    let mut grapheme_index = 0;
                    let mut column = 0;
                    for grapheme_width in previous_row.widths() {
                        if column + grapheme_width > cursor_column {
                            break;
                        }

                        column += grapheme_width;
                        grapheme_index += 1;
                    }

                    self.cursor = self.cursor.add_y(-1).set_x(grapheme_index);
                }
            }
            Key::Down => {
                if self.cursor.y() + 1 < self.get_total_row_count() {
                    let cursor_column = self.get_cursor_column();
                    let next_row = self.get_row(self.cursor.y() + 1);

                    let mut grapheme_index = 0;
                    let mut column = 0;
                    for grapheme_width in next_row.widths() {
                        if column + grapheme_width > cursor_column {
                            break;
                        }

                        column += grapheme_width;
                        grapheme_index += 1;
                    }

                    self.cursor = self.cursor.add_y(1).set_x(grapheme_index);
                }
            }
            Key::Backspace => {
                let (line_index, grapheme_index) = self.get_cursor_line_and_grapheme_indexes();
                if self.cursor.x() > 0 {
                    let line = &self.lines[line_index];
                    let (index, length) =
                        self.get_grapheme_byte_information(line, grapheme_index - 1);
                    let length = length.expect("Grapheme should have byte length");

                    self.lines[line_index].replace_range(index..index + length, "");

                    self.cursor = self.cursor.add_x(-1);
                } else {
                    if self.cursor.y() > 0 {
                        let prior_row_length = self.get_row_length(self.cursor.y() - 1);
                        self.cursor = self.cursor.add_y(-1).set_x(prior_row_length);

                        let line_content = self.lines[line_index].clone();
                        self.lines[line_index - 1].push_str(&line_content);
                        self.lines.remove(line_index);
                    }
                }
            }
            Key::Delete => {
                let row_length = self.get_row_length(self.cursor.y());
                if self.cursor.x() < row_length {
                    let (line_index, grapheme_index) = self.get_cursor_line_and_grapheme_indexes();
                    let line = &self.lines[line_index];
                    let (index, length) =
                        self.get_grapheme_byte_information(line, grapheme_index + 1);
                    let length = length.expect("Grapheme should have byte length");

                    let line = &mut self.lines[line_index];
                    line.replace_range(index..index + length, "");
                }
            }
            Key::Enter => {
                let (line_index, grapheme_index) = self.get_cursor_line_and_grapheme_indexes();
                let line = &self.lines[line_index];
                let (byte_index, _) = self.get_grapheme_byte_information(line, grapheme_index);

                let trailing_content = line[byte_index..].to_string();
                self.lines.insert(line_index + 1, trailing_content);
                self.cursor = self.cursor.add_y(1).set_x(0);

                self.lines[line_index].replace_range(byte_index.., "");
            }
            Key::Home => {
                self.cursor = self.cursor.set_x(0);
            }
            Key::End => {
                let row_length = self.get_row_length(self.cursor.y());
                self.cursor = self.cursor.set_x(row_length);
            }
        }
    }

    /// Returns the specified row's length as a grapheme count.
    fn get_row_length(&self, row_index: usize) -> usize {
        self.get_row(row_index).widths().len()
    }

    /// Returns the specified row.
    fn get_row(&self, row_index: usize) -> &RowLayout {
        let mut row = 0;
        for line_layout in &self.layout {
            if row + line_layout.rows().len() > row_index {
                return &line_layout.rows()[row_index - row];
            } else {
                row += line_layout.rows().len();
            }
        }

        panic!("Invalid row index: {}", row_index);
    }

    /// Returns the column position of the cursor.
    fn get_cursor_column(&self) -> usize {
        let mut grapheme = 0;
        let mut column = 0;

        for width in self.get_row(self.cursor.y()).widths() {
            if grapheme == self.cursor.x() {
                break;
            }

            column += width;
            grapheme += 1;
        }

        return column;
    }

    /// Returns the total row count.
    fn get_total_row_count(&self) -> usize {
        self.layout
            .iter()
            .map(|line_layout| line_layout.rows().len())
            .sum()
    }

    /// Returns the cursor's corresponding line and grapheme index.
    fn get_cursor_line_and_grapheme_indexes(&self) -> (usize, usize) {
        if self.cursor == Position::default() {
            return (0, 0);
        }

        let mut row = 0;

        for (line_index, line_layout) in self.layout.iter().enumerate() {
            let line_row_count = line_layout.rows().len();

            if row + line_row_count > self.cursor.y() {
                let mut grapheme_count = 0;
                for row_layout in line_layout.rows() {
                    if row == self.cursor.y() {
                        return (line_index, grapheme_count + self.cursor.x());
                    }

                    grapheme_count += row_layout.widths().len();
                    row += 1;
                }
            } else {
                row += line_row_count;
            }
        }

        panic!("Cursor did not intersect any row.");
    }

    /// Returns the byte index and length for the specified grapheme. Length unspecified if N/A.
    fn get_grapheme_byte_information(
        &self,
        line: &str,
        grapheme_index: usize,
    ) -> (usize, Option<usize>) {
        if line.is_empty() {
            return (0, None);
        }

        let grapheme_info = line.grapheme_indices(true).skip(grapheme_index).next();

        if let Some(grapheme_info) = grapheme_info {
            (grapheme_info.0, Some(grapheme_info.1.len()))
        } else {
            (line.len(), None)
        }
    }
}
