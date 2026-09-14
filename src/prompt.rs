const SCROLLOFF: usize = 4;

#[derive(Debug)]
pub enum PromptKind {
    Open,
    SaveAs,
}

#[derive(Debug)]
pub struct Prompt {
    buffer: String,
    kind: PromptKind,
    cursor: usize,
    viewport_left: usize,
    viewport_cols: usize,
}

impl Prompt {
    pub fn new(kind: PromptKind) -> Self {
        Self {
            buffer: String::new(),
            kind,
            cursor: 0,
            viewport_left: 0,
            viewport_cols: 0,
        }
    }

    pub fn prompt(&self) -> &str {
        &self.buffer
    }

    pub fn kind(&self) -> &PromptKind {
        &self.kind
    }

    pub fn cursor_pos(&self) -> usize {
        self.char_column(self.cursor) + self.label().chars().count()
    }

    pub fn viewport(&self) -> (usize, usize) {
        (self.viewport_left, self.viewport_cols)
    }

    pub fn scroll_to_cursor(&mut self) {
        let col = self.char_column(self.cursor);

        if col.saturating_sub(SCROLLOFF) < self.viewport_left {
            self.viewport_left = col.saturating_sub(SCROLLOFF);
        } else if col.saturating_add(SCROLLOFF) >= self.viewport_left + self.viewport_cols {
            self.viewport_left = col.saturating_add(SCROLLOFF) - self.viewport_cols + 1;
        }
    }

    pub fn set_viewport_size(&mut self, cols: usize) {
        self.viewport_cols = cols.saturating_sub(self.label().chars().count());
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.buffer.len();
    }

    pub fn move_left(&mut self) {
        self.cursor = self.prev_boundary(self.cursor);
    }

    pub fn move_right(&mut self) {
        self.cursor = self.next_boundary(self.cursor);
    }

    pub fn insert(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            let start = self.prev_boundary(self.cursor);
            self.buffer.remove(start);
            self.cursor = start;
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
        }
    }

    pub fn visible_line(&self) -> String {
        let start = self.advance_chars(0, self.viewport_left, self.buffer.len());
        let end = self.advance_chars(start, self.viewport_cols, self.buffer.len());

        let visible_prompt = self.buffer[start..end].to_string();
        format!("{}{}", self.label(), visible_prompt)
    }

    fn label(&self) -> &'static str {
        match self.kind {
            PromptKind::Open => "open: ",
            PromptKind::SaveAs => "save as: ",
        }
    }

    fn char_column(&self, offset: usize) -> usize {
        self.buffer[..offset].chars().count()
    }

    fn advance_chars(&self, offset: usize, n: usize, limit: usize) -> usize {
        if offset >= limit {
            return offset;
        }

        self.buffer[offset..limit]
            .chars()
            .take(n)
            .fold(offset, |acc, curr| acc + curr.len_utf8())
    }

    fn next_boundary(&self, offset: usize) -> usize {
        if offset >= self.buffer.len() {
            return offset;
        }

        let mut i = offset + 1;
        while !self.buffer.is_char_boundary(i) {
            i += 1;
        }

        i
    }

    fn prev_boundary(&self, offset: usize) -> usize {
        if offset == 0 {
            return offset;
        }

        let mut i = offset - 1;
        while !self.buffer.is_char_boundary(i) {
            i -= 1;
        }

        i
    }
}
