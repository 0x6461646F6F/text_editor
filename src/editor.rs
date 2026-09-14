use crate::rope::Rope;

const SCROLLOFF: usize = 4;

#[derive(Debug)]
pub struct Editor {
    buffer: Rope,
    cursor: usize,
    preferred_col: Option<usize>,
    line_count: usize,
    viewport_left: usize,
    viewport_top: usize,
    viewport_cols: usize,
    viewport_rows: usize,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            buffer: Rope::default(),
            cursor: 0,
            preferred_col: None,
            line_count: 1,
            viewport_left: 0,
            viewport_top: 0,
            viewport_cols: 0,
            viewport_rows: 0,
        }
    }
}

impl From<&str> for Editor {
    fn from(value: &str) -> Self {
        Self {
            line_count: value.matches('\n').count() + 1,
            buffer: Rope::from(value),
            ..Self::default()
        }
    }
}

impl From<String> for Editor {
    fn from(value: String) -> Self {
        Self {
            line_count: value.matches('\n').count() + 1,
            buffer: Rope::from(value),
            ..Self::default()
        }
    }
}

impl Editor {
    pub fn text(&self) -> String {
        self.buffer.to_string()
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.char_column(self.cursor), self.line_number(self.cursor))
    }

    pub fn line_count(&self) -> usize {
        self.line_count
    }

    pub fn viewport(&self) -> (usize, usize, usize, usize) {
        (
            self.viewport_left,
            self.viewport_top,
            self.viewport_cols,
            self.viewport_rows,
        )
    }

    pub fn set_viewport_size(&mut self, cols: usize, rows: usize) {
        self.viewport_cols = cols;
        self.viewport_rows = rows;
    }

    pub fn scroll_to_cursor(&mut self) {
        let col = self.char_column(self.cursor);
        let row = self.line_number(self.cursor);

        if col.saturating_sub(SCROLLOFF) < self.viewport_left {
            self.viewport_left = col.saturating_sub(SCROLLOFF);
        } else if col.saturating_add(SCROLLOFF) >= self.viewport_left + self.viewport_cols {
            self.viewport_left = col.saturating_add(SCROLLOFF) - self.viewport_cols + 1;
        }

        if row.saturating_sub(SCROLLOFF) < self.viewport_top {
            self.viewport_top = row.saturating_sub(SCROLLOFF);
        } else if row.saturating_add(SCROLLOFF) >= self.viewport_top + self.viewport_rows {
            self.viewport_top = row.saturating_add(SCROLLOFF) - self.viewport_rows + 1;
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = self.line_start(self.cursor);
        self.preferred_col = None;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.line_end(self.cursor);
        self.preferred_col = None;
    }

    pub fn move_home_full(&mut self) {
        self.cursor = 0;
        self.preferred_col = None;
    }

    pub fn move_end_full(&mut self) {
        self.cursor = self.buffer.len();
        self.preferred_col = None;
    }

    pub fn move_left(&mut self) {
        self.cursor = self.prev_boundary(self.cursor);
        self.preferred_col = None;
    }

    pub fn move_right(&mut self) {
        self.cursor = self.next_boundary(self.cursor);
        self.preferred_col = None;
    }

    pub fn move_up(&mut self) {
        let start = self.line_start(self.cursor);

        if start == 0 {
            self.cursor = start;
            return;
        }

        let prev_start = self.line_start(start - 1);
        let prev_end = start - 1;

        let col = match self.preferred_col {
            Some(c) => c,
            None => {
                let c = self.char_column(self.cursor);
                self.preferred_col = Some(c);
                c
            }
        };

        self.cursor = self.advance_chars(prev_start, col, prev_end);
    }

    pub fn move_down(&mut self) {
        let end = self.line_end(self.cursor);

        if end == self.buffer.len() {
            self.cursor = end;
            return;
        }

        let next_start = end + 1;
        let next_end = self.line_end(next_start);

        let col = match self.preferred_col {
            Some(c) => c,
            None => {
                let c = self.char_column(self.cursor);
                self.preferred_col = Some(c);
                c
            }
        };

        self.cursor = self.advance_chars(next_start, col, next_end);
    }

    pub fn move_word_left(&mut self) {
        let mut prev_offset = self.prev_boundary(self.cursor);

        while self.cursor > 0 && !self.is_word_char(prev_offset) {
            self.cursor = prev_offset;
            prev_offset = self.prev_boundary(self.cursor);
        }

        while self.cursor > 0 && self.is_word_char(prev_offset) {
            self.cursor = prev_offset;
            prev_offset = self.prev_boundary(self.cursor);
        }
    }

    pub fn move_word_right(&mut self) {
        while self.cursor < self.buffer.len() && !self.is_word_char(self.cursor) {
            self.cursor = self.next_boundary(self.cursor);
        }

        while self.cursor < self.buffer.len() && self.is_word_char(self.cursor) {
            self.cursor = self.next_boundary(self.cursor);
        }
    }

    pub fn insert(&mut self, c: char) {
        self.buffer = self.buffer.clone().insert_char(self.cursor, c);
        self.cursor += c.len_utf8();
        self.preferred_col = None;

        if c == '\n' {
            self.line_count += 1;
        }
    }

    pub fn insert_str(&mut self, s: &str) {
        self.buffer = self.buffer.clone().insert_str(self.cursor, s);
        self.cursor += s.len();
        self.preferred_col = None;
        self.line_count += s.matches('\n').count();
    }

    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            let start = self.prev_boundary(self.cursor);
            let byte = self.buffer.byte_at(start);

            self.buffer = self.buffer.clone().remove(start, self.cursor - start);
            self.cursor = start;
            self.preferred_col = None;

            if let Some(b'\n') = byte {
                self.line_count -= 1;
            }
        }

        assert!(self.line_count >= 1, "line_count can't be less than 1");
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.buffer.len() {
            let end = self.next_boundary(self.cursor);
            let byte = self.buffer.byte_at(self.cursor);

            self.buffer = self.buffer.clone().remove(self.cursor, end - self.cursor);

            if let Some(b'\n') = byte {
                self.line_count -= 1;
            }
        }

        assert!(self.line_count >= 1, "line_count can't be less than 1");
    }

    pub fn delete_line(&mut self) {
        let start = self.line_start(self.cursor);
        let end = self.line_end(self.cursor);
        println!("{}", self.line_count);

        if end == self.buffer.len() {
            let new_start = start.saturating_sub(1);
            self.buffer = self.buffer.clone().remove(new_start, end - new_start);
            self.cursor = new_start;

            if start > 0 {
                self.line_count -= 1;
            }
        } else {
            let new_end = end + 1;
            self.buffer = self.buffer.clone().remove(start, new_end - start);
            self.cursor = start;
            self.line_count -= 1;
        }

        self.preferred_col = None;

        assert!(self.line_count >= 1, "line_count can't be less than 1");
    }

    pub fn visible_lines(&self) -> impl Iterator<Item = String> {
        let mut curr_start = 0;
        let mut curr_line = 0;

        std::iter::from_fn(move || {
            while curr_line < self.viewport_top && curr_start < self.buffer.len() {
                curr_start = self.line_end(curr_start) + 1;
                curr_line += 1;
            }

            while curr_line >= self.viewport_top
                && curr_line < self.viewport_top + self.viewport_rows
                && curr_start <= self.buffer.len()
            {
                let curr_end = self.line_end(curr_start);

                let start = self.advance_chars(curr_start, self.viewport_left, curr_end);
                let end = self.advance_chars(start, self.viewport_cols, curr_end);

                let s = self.buffer.slice_to_string(start, end - start);

                curr_start = curr_end + 1;
                curr_line += 1;

                return Some(s);
            }

            None
        })
    }

    fn char_column(&self, offset: usize) -> usize {
        let start = self.line_start(offset);

        self.buffer
            .slice_to_string(start, offset - start)
            .chars()
            .count()
    }

    fn line_number(&self, offset: usize) -> usize {
        self.buffer
            .slice_to_string(0, offset)
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
    }

    fn line_start(&self, offset: usize) -> usize {
        self.buffer
            .slice_to_string(0, offset)
            .rfind('\n')
            .map_or_default(|i| i + 1)
    }

    fn line_end(&self, offset: usize) -> usize {
        self.buffer
            .slice_to_string(offset, self.buffer.len() - offset)
            .find('\n')
            .map_or(self.buffer.len(), |i| i + offset)
    }

    fn advance_chars(&self, offset: usize, n: usize, limit: usize) -> usize {
        if offset >= limit {
            return offset;
        }

        self.buffer
            .slice_to_string(offset, limit - offset)
            .chars()
            .take(n)
            .fold(offset, |acc, curr| acc + curr.len_utf8())
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.buffer
            .next_char_boundary(offset)
            .unwrap_or(self.buffer.len())
    }

    fn prev_boundary(&self, offset: usize) -> usize {
        self.buffer.prev_char_boundary(offset).unwrap_or_default()
    }

    fn is_word_char(&self, offset: usize) -> bool {
        self.buffer
            .char_at(offset)
            .is_some_and(|c| c.is_alphanumeric() || c == '_')
    }
}
