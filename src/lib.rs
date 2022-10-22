//! # tty-text
//!
//! Provides agnostic text editing state management.
//!
//! # Examples
//! For more examples, see [Text].
//! ```
//! use tty_text::{Text, Key};
//!
//! let mut text = Text::from("Hello,\nworld!", (1, 0), true);
//!
//! // Move cursor from "e" to "w"
//! text.handle_input(Key::Down);
//! text.handle_input(Key::Down);
//! text.handle_input(Key::Left);
//!
//! // Combine into single line, add " "
//! text.handle_input(Key::Backspace);
//! text.handle_input(Key::Char(' '));
//!
//! // Add another "!"
//! text.set_cursor((13, 0));
//! text.handle_input(Key::Char('!'));
//!
//! assert_eq!("Hello, world!!", text.value());
//! assert_eq!((14, 0), text.cursor());
//! ```

pub enum Key {
    Char(char),
    Backspace,
    Enter,
    Up,
    Down,
    Left,
    Right,
}

/// A multi-line text editor with cursor management capabilities.
///
/// # Examples
/// ## Single-line mode
/// ```
/// use tty_text::{Text, Key};
///
/// let mut text = Text::new(false);
///
/// text.handle_input(Key::Char('a'));
/// text.handle_input(Key::Enter);
/// text.handle_input(Key::Char('b'));
///
/// assert_eq!((2, 0), text.cursor());
/// assert_eq!("ab", text.value());
/// assert_eq!(&vec![
///     "ab".to_string(),
/// ], text.lines());
/// ```
/// ## Multi-line mode
/// ```
/// use tty_text::{Text, Key};
///
/// let mut text = Text::new(true);
///
/// text.handle_input(Key::Char('a'));
/// text.handle_input(Key::Enter);
/// text.handle_input(Key::Char('b'));

/// assert_eq!((1, 1), text.cursor());
/// assert_eq!("a\nb", text.value());
/// assert_eq!(&vec![
///     "a".to_string(),
///     "b".to_string(),
/// ], text.lines());
/// ```
pub struct Text {
    /// The lines that comprise this editor's value.
    lines: Vec<String>,

    /// The cursor's position in the editor in (columns, lines).
    cursor: (usize, usize),

    /// Whether this editor is configured for multi-line value editing.
    multi_line: bool,
}

impl Text {
    /// Create a new, empty editor in the specified mode.
    ///
    /// # Examples
    /// ```
    /// use tty_text::{Text, Key};
    ///
    /// let mut text = Text::new(false);
    ///
    /// text.handle_input(Key::Char('a'));
    /// text.handle_input(Key::Char('b'));
    /// text.handle_input(Key::Char('d'));
    /// text.set_cursor((2, 0));
    /// text.handle_input(Key::Char('c'));
    ///
    /// assert_eq!((3, 0), text.cursor());
    /// assert_eq!("abcd", text.value());
    /// ```
    pub fn new(multi_line: bool) -> Self {
        Self {
            lines: vec![String::new()],
            cursor: (0, 0),
            multi_line,
        }
    }

    /// Create a new editor from the specified value and cursor state and in the specified mode.
    ///
    /// # Examples
    /// ## Multi-line value and mode
    /// ```
    /// use tty_text::{Text, Key};
    ///
    /// let mut text = Text::from("Hello,\nworld!", (2, 1), true);
    ///
    /// assert_eq!("Hello,\nworld!", text.value());
    /// assert_eq!((2, 1), text.cursor());
    /// assert_eq!(&vec![
    ///     "Hello,".to_string(),
    ///     "world!".to_string(),
    /// ], text.lines());
    /// ```
    /// ## Multi-line value collapsed by single-line mode
    /// ```
    /// use tty_text::{Text, Key};
    ///
    /// let mut text = Text::from("Hello,\n world!", (7, 1), false);
    ///
    /// assert_eq!("Hello, world!", text.value());
    /// assert_eq!((7, 0), text.cursor());
    /// assert_eq!(&vec![
    ///     "Hello, world!".to_string(),
    /// ], text.lines());
    /// ```
    pub fn from(value: &str, cursor: (usize, usize), multi_line: bool) -> Self {
        let mut lines = if multi_line {
            value.lines().map(|line| line.to_string()).collect()
        } else {
            vec![value.replace("\n", "").replace("\r", "")]
        };

        if lines.is_empty() || value.ends_with("\n") || value.ends_with("\r\n") {
            lines.push(String::new());
        }

        let mut text = Self {
            lines,
            cursor: (0, 0),
            multi_line,
        };

        text.set_cursor(cursor);

        text
    }

    /// This editor's current cursor position as (columns, lines).
    pub fn cursor(&self) -> (usize, usize) {
        self.cursor
    }

    /// This editor's current value.
    pub fn value(&self) -> String {
        self.lines.join("\n")
    }

    /// This editor's value's lines.
    pub fn lines(&self) -> &Vec<String> {
        &self.lines
    }

    /// Update this editor's cursor position. The position will be clamped to the editor's current
    /// value.
    pub fn set_cursor(&mut self, position: (usize, usize)) {
        self.cursor = position;

        // Clamp the line
        if self.cursor.1 >= self.lines.len() {
            self.cursor.1 = self.lines.len() - 1;
        }

        // Clamp the column
        let line_length = self.get_line_length(self.cursor.1);
        if self.cursor.0 > line_length {
            self.cursor.0 = line_length;
        }
    }

    /// Update this editor's state from the specified input.
    pub fn handle_input(&mut self, input: Key) {
        match input {
            Key::Char(ch) => self.insert_character(ch),
            Key::Backspace => self.backspace_character(),
            Key::Enter => self.insert_newline(),
            Key::Up => self.move_up(),
            Key::Down => self.move_down(),
            Key::Left => self.move_left(),
            Key::Right => self.move_right(),
        }
    }

    /// Insert the specified character at the editor's current cursor position.
    fn insert_character(&mut self, ch: char) {
        self.lines[self.cursor.1].insert(self.cursor.0, ch);
        self.cursor.0 += 1;
    }

    /// Backspace the character preceding the editor's current cursor position.
    fn backspace_character(&mut self) {
        let at_start_of_line = self.cursor.0 == 0;
        if at_start_of_line {
            let on_first_line = self.cursor.1 == 0;
            if !on_first_line {
                // Remove the current line
                let line = self.lines.remove(self.cursor.1);

                // Move the cursor to the end of the previous line
                let prior_line_index = self.cursor.1 - 1;
                self.cursor = (self.get_line_length(prior_line_index), prior_line_index);

                // Append the just-deleted line after the cursor in the previous line
                self.lines[self.cursor.1].push_str(&line);
            }
        } else {
            self.cursor.0 -= 1;
            self.lines[self.cursor.1].remove(self.cursor.0);
        }
    }

    /// Insert a newline at the editor's current cursor position.
    fn insert_newline(&mut self) {
        if !self.multi_line {
            return;
        }

        // Split the current line at the cursor
        let (prefix, suffix) = self.lines[self.cursor.1].split_at(self.cursor.0).to_owned();
        let (prefix, suffix) = (prefix.to_string(), suffix.to_string());

        // Shorten the current line to the content preceding the cursor
        self.lines[self.cursor.1] = prefix;

        // Insert a new line after the current one with the content after the cursor
        let new_line_index = self.cursor.1 + 1;
        self.lines.insert(new_line_index, suffix);

        // Move the cursor to the start of the next line
        self.cursor = (0, new_line_index);
    }

    /// Attempt to move the editor's cursor up one line.
    fn move_up(&mut self) {
        if !self.multi_line {
            return;
        }

        let on_first_line = self.cursor.1 == 0;
        if !on_first_line {
            let previous_line = self.cursor.1 - 1;
            let new_column = std::cmp::min(self.cursor.0, self.get_line_length(previous_line));
            self.cursor = (new_column, previous_line);
        }
    }

    /// Attempt to move the editor's cursor down one line.
    fn move_down(&mut self) {
        if !self.multi_line {
            return;
        }

        let next_line = self.cursor.1 + 1;

        let is_last_line = next_line == self.lines.len();
        if !is_last_line {
            let new_column = std::cmp::min(self.cursor.0, self.get_line_length(next_line));
            self.cursor = (new_column, self.cursor.1 + 1);
        }
    }

    /// Attempt to move the editor's cursor left one character.
    fn move_left(&mut self) {
        let at_start_of_line = self.cursor.0 == 0;
        let on_first_line = self.cursor.1 == 0;

        if !at_start_of_line {
            self.cursor.0 -= 1;
        } else if !on_first_line {
            let previous_line = self.cursor.1 - 1;
            self.cursor = (self.get_line_length(previous_line), previous_line);
        }
    }

    /// Attempt to move the editor's cursor right one character.
    fn move_right(&mut self) {
        let at_end_of_line = self.cursor.0 == self.get_line_length(self.cursor.1);
        let on_last_line = self.cursor.1 + 1 == self.lines.len();

        if !at_end_of_line {
            self.cursor.0 += 1;
        } else if !on_last_line {
            self.cursor = (0, self.cursor.1 + 1);
        }
    }

    /// Get the specified line's length.
    fn get_line_length(&self, line_index: usize) -> usize {
        self.lines[line_index].len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! svec {
        ($($x:expr),*) => (vec![$($x.to_string()),*]);
    }

    macro_rules! assert_text {
        ($text: ident, $cursor: expr, $value: expr, $lines: expr) => {
            assert_eq!($cursor, $text.cursor());
            assert_eq!($value, $text.value());
            assert_eq!(&$lines, $text.lines());
        };
    }

    #[test]
    fn new() {
        let text = Text::new(false);
        assert_text!(text, (0, 0), "", svec![""]);
    }

    #[test]
    fn from() {
        let text = Text::from("a\nbc", (1, 1), true);
        assert_text!(text, (1, 1), "a\nbc", svec!["a", "bc"]);
    }

    #[test]
    fn from_clamp_cursor() {
        let text = Text::from("a\nbc", (5, 5), true);
        assert_text!(text, (2, 1), "a\nbc", svec!["a", "bc"]);
    }

    #[test]
    fn from_collapse_single_line() {
        let text = Text::from("a\n\nbc", (1, 0), false);
        assert_text!(text, (1, 0), "abc", svec!["abc"]);
    }

    #[test]
    fn from_clamp_cursor_single_line() {
        let text = Text::from("a\n\nbc", (3, 0), false);
        assert_text!(text, (3, 0), "abc", svec!["abc"]);
    }

    #[test]
    fn blank_lines() {
        let text = Text::from("abc\n\r\n", (0, 1), true);
        assert_text!(text, (0, 1), "abc\n\n", svec!["abc", "", ""]);
    }

    #[test]
    fn set_cursor() {
        let mut text = Text::from("a\nbc", (0, 0), true);
        assert_eq!((0, 0), text.cursor());

        text.set_cursor((1, 1));
        assert_eq!((1, 1), text.cursor());
    }

    #[test]
    fn set_cursor_clamping() {
        let mut text = Text::from("a\nbc", (0, 0), true);
        assert_eq!((0, 0), text.cursor());

        text.set_cursor((5, 5));
        assert_eq!((2, 1), text.cursor());
    }

    #[test]
    fn handle_input() {
        let mut text = Text::from("abc\ndef", (2, 1), true);
        assert_text!(text, (2, 1), "abc\ndef", svec!["abc", "def"]);

        text.handle_input(Key::Char('X'));
        assert_text!(text, (3, 1), "abc\ndeXf", svec!["abc", "deXf"]);

        text.handle_input(Key::Left);
        assert_text!(text, (2, 1), "abc\ndeXf", svec!["abc", "deXf"]);

        text.handle_input(Key::Backspace);
        assert_text!(text, (1, 1), "abc\ndXf", svec!["abc", "dXf"]);

        text.handle_input(Key::Right);
        text.handle_input(Key::Right);
        assert_text!(text, (3, 1), "abc\ndXf", svec!["abc", "dXf"]);

        text.handle_input(Key::Char('g'));
        text.handle_input(Key::Char('h'));
        text.handle_input(Key::Char('i'));
        assert_text!(text, (6, 1), "abc\ndXfghi", svec!["abc", "dXfghi"]);

        text.handle_input(Key::Up);
        assert_text!(text, (3, 0), "abc\ndXfghi", svec!["abc", "dXfghi"]);

        text.handle_input(Key::Left);
        text.handle_input(Key::Left);
        assert_text!(text, (1, 0), "abc\ndXfghi", svec!["abc", "dXfghi"]);

        text.handle_input(Key::Down);
        assert_text!(text, (1, 1), "abc\ndXfghi", svec!["abc", "dXfghi"]);

        text.handle_input(Key::Backspace);
        text.handle_input(Key::Backspace);
        assert_text!(text, (3, 0), "abcXfghi", svec!["abcXfghi"]);

        text.handle_input(Key::Right);
        text.handle_input(Key::Right);
        assert_text!(text, (5, 0), "abcXfghi", svec!["abcXfghi"]);

        text.handle_input(Key::Enter);
        assert_text!(text, (0, 1), "abcXf\nghi", svec!["abcXf", "ghi"]);

        text.handle_input(Key::Left);
        assert_text!(text, (5, 0), "abcXf\nghi", svec!["abcXf", "ghi"]);

        text.handle_input(Key::Down);
        assert_text!(text, (3, 1), "abcXf\nghi", svec!["abcXf", "ghi"]);
    }

    #[test]
    fn handle_input_single_line() {
        let mut text = Text::from("abcdef", (3, 0), false);
        assert_text!(text, (3, 0), "abcdef", svec!["abcdef"]);

        text.handle_input(Key::Char('X'));
        assert_text!(text, (4, 0), "abcXdef", svec!["abcXdef"]);

        text.handle_input(Key::Enter);
        assert_text!(text, (4, 0), "abcXdef", svec!["abcXdef"]);

        text.handle_input(Key::Up);
        assert_text!(text, (4, 0), "abcXdef", svec!["abcXdef"]);

        text.handle_input(Key::Down);
        assert_text!(text, (4, 0), "abcXdef", svec!["abcXdef"]);

        text.handle_input(Key::Backspace);
        assert_text!(text, (3, 0), "abcdef", svec!["abcdef"]);

        text.set_cursor((0, 0));
        text.move_left();
        assert_text!(text, (0, 0), "abcdef", svec!["abcdef"]);

        text.set_cursor((6, 0));
        text.move_right();
        assert_text!(text, (6, 0), "abcdef", svec!["abcdef"]);
    }

    #[test]
    fn insert_character_end_line() {
        let mut text = Text::new(true);

        text.insert_character('a');
        text.insert_character('b');
        text.insert_character('c');

        assert_text!(text, (3, 0), "abc", svec!["abc"]);
    }

    #[test]
    fn insert_character_mid_line() {
        let mut text = Text::from("abc", (1, 0), true);

        text.insert_character('X');

        assert_text!(text, (2, 0), "aXbc", svec!["aXbc"]);
    }

    #[test]
    fn insert_character_start_line() {
        let mut text = Text::from("abc", (0, 0), true);

        text.insert_character('X');

        assert_text!(text, (1, 0), "Xabc", svec!["Xabc"]);
    }

    #[test]
    fn backspace_character_all() {
        let mut text = Text::from("abc", (3, 0), true);

        text.backspace_character();
        text.backspace_character();
        text.backspace_character();

        assert_text!(text, (0, 0), "", svec![""]);
    }

    #[test]
    fn backspace_character_mid_line() {
        let mut text = Text::from("abc", (2, 0), true);

        text.backspace_character();
        text.backspace_character();
        text.backspace_character();

        assert_text!(text, (0, 0), "c", svec!["c"]);
    }

    #[test]
    fn backspace_character_start_line() {
        let mut text = Text::from("abc", (0, 0), true);

        text.backspace_character();
        text.backspace_character();
        text.backspace_character();

        assert_text!(text, (0, 0), "abc", svec!["abc"]);
    }

    #[test]
    fn backspace_character_multi_line() {
        let mut text = Text::from("abc\ndef", (0, 1), true);

        text.backspace_character();

        assert_text!(text, (3, 0), "abcdef", svec!["abcdef"]);
    }

    #[test]
    fn insert_newline_end_line() {
        let mut text = Text::from("abc", (3, 0), true);

        text.insert_newline();

        assert_text!(text, (0, 1), "abc\n", svec!["abc", ""]);
    }

    #[test]
    fn insert_newline_start_line() {
        let mut text = Text::from("abc", (0, 0), true);

        text.insert_newline();

        assert_text!(text, (0, 1), "\nabc", svec!["", "abc"]);
    }

    #[test]
    fn insert_newline_mid_line() {
        let mut text = Text::from("abc", (1, 0), true);

        text.insert_newline();

        assert_text!(text, (0, 1), "a\nbc", svec!["a", "bc"]);
    }

    #[test]
    fn insert_newline_empty() {
        let mut text = Text::from("", (0, 0), true);

        text.insert_newline();

        assert_text!(text, (0, 1), "\n", svec!["", ""]);
    }

    #[test]
    fn insert_newline_single_line() {
        let mut text = Text::from("abcdef", (3, 0), false);

        text.insert_newline();

        assert_text!(text, (3, 0), "abcdef", svec!["abcdef"]);
    }

    #[test]
    fn move_up_start_line() {
        let mut text = Text::from("abc\ndef", (0, 1), true);

        text.move_up();

        assert_text!(text, (0, 0), "abc\ndef", svec!["abc", "def"]);
    }

    #[test]
    fn move_up_mid_line() {
        let mut text = Text::from("abc\ndef", (2, 1), true);

        text.move_up();

        assert_text!(text, (2, 0), "abc\ndef", svec!["abc", "def"]);
    }

    #[test]
    fn move_up_end_line() {
        let mut text = Text::from("abc\ndef", (3, 1), true);

        text.move_up();

        assert_text!(text, (3, 0), "abc\ndef", svec!["abc", "def"]);
    }

    #[test]
    fn move_up_shorter_line() {
        let mut text = Text::from("a\ndef", (2, 1), true);

        text.move_up();

        assert_text!(text, (1, 0), "a\ndef", svec!["a", "def"]);
    }

    #[test]
    fn move_up_single_line() {
        let mut text = Text::from("abcdef", (3, 0), true);

        text.move_up();

        assert_text!(text, (3, 0), "abcdef", svec!["abcdef"]);
    }

    #[test]
    fn move_down_start_line() {
        let mut text = Text::from("abc\ndef", (0, 0), true);

        text.move_down();

        assert_text!(text, (0, 1), "abc\ndef", svec!["abc", "def"]);
    }

    #[test]
    fn move_down_mid_line() {
        let mut text = Text::from("abc\ndef", (2, 0), true);

        text.move_down();

        assert_text!(text, (2, 1), "abc\ndef", svec!["abc", "def"]);
    }

    #[test]
    fn move_down_end_line() {
        let mut text = Text::from("abc\ndef", (3, 0), true);

        text.move_down();

        assert_text!(text, (3, 1), "abc\ndef", svec!["abc", "def"]);
    }

    #[test]
    fn move_down_shorter_line() {
        let mut text = Text::from("a\ndef", (2, 0), true);

        text.move_down();

        assert_text!(text, (1, 1), "a\ndef", svec!["a", "def"]);
    }

    #[test]
    fn move_down_single_line() {
        let mut text = Text::from("abcdef", (3, 0), false);

        text.move_down();

        assert_text!(text, (3, 0), "abcdef", svec!["abcdef"]);
    }

    #[test]
    fn move_left_mid_line() {
        let mut text = Text::from("abc", (2, 0), true);

        text.move_left();

        assert_text!(text, (1, 0), "abc", svec!["abc"]);
    }

    #[test]
    fn move_left_end_line() {
        let mut text = Text::from("abc", (3, 0), true);

        text.move_left();

        assert_text!(text, (2, 0), "abc", svec!["abc"]);
    }

    #[test]
    fn move_left_start_value() {
        let mut text = Text::from("abc", (0, 0), true);

        text.move_left();

        assert_text!(text, (0, 0), "abc", svec!["abc"]);
    }

    #[test]
    fn move_left_wrap_up() {
        let mut text = Text::from("abc\ndef", (0, 1), true);

        text.move_left();

        assert_text!(text, (3, 0), "abc\ndef", svec!["abc", "def"]);
    }

    #[test]
    fn move_left_wrap_up_empty_line() {
        let mut text = Text::from("abc\n\ndef", (0, 2), true);

        text.move_left();

        assert_text!(text, (0, 1), "abc\n\ndef", svec!["abc", "", "def"]);
    }

    #[test]
    fn move_left_single_line() {
        let mut text = Text::from("abcdef", (0, 0), false);

        text.move_left();

        assert_text!(text, (0, 0), "abcdef", svec!["abcdef"]);
    }

    #[test]
    fn move_right_mid_line() {
        let mut text = Text::from("abc", (1, 0), true);

        text.move_right();

        assert_text!(text, (2, 0), "abc", svec!["abc"]);
    }

    #[test]
    fn move_right_start_line() {
        let mut text = Text::from("abc", (0, 0), true);

        text.move_right();

        assert_text!(text, (1, 0), "abc", svec!["abc"]);
    }

    #[test]
    fn move_right_end_value() {
        let mut text = Text::from("abc", (3, 0), true);

        text.move_right();

        assert_text!(text, (3, 0), "abc", svec!["abc"]);
    }

    #[test]
    fn move_right_wrap_down() {
        let mut text = Text::from("abc\ndef", (3, 0), true);

        text.move_right();

        assert_text!(text, (0, 1), "abc\ndef", svec!["abc", "def"]);
    }

    #[test]
    fn move_right_wrap_down_empty_line() {
        let mut text = Text::from("abc\n\ndef", (3, 0), true);

        text.move_right();

        assert_text!(text, (0, 1), "abc\n\ndef", svec!["abc", "", "def"]);
    }

    #[test]
    fn move_right_single_line() {
        let mut text = Text::from("abcdef", (6, 0), false);

        text.move_right();

        assert_text!(text, (6, 0), "abcdef", svec!["abcdef"]);
    }
}
