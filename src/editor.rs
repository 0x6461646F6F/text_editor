#[derive(Debug, Default)]
pub struct Editor {
    buffer: String,
    cursor: usize,
    preferred_col: Option<usize>,
    viewport_top: usize,
    viewport_left: usize,
    viewport_rows: usize,
    viewport_cols: usize,
}

impl From<&str> for Editor {
    fn from(value: &str) -> Self {
        Self {
            buffer: value.to_string(),
            ..Self::new()
        }
    }
}

impl From<String> for Editor {
    fn from(value: String) -> Self {
        Self {
            buffer: value,
            ..Self::new()
        }
    }
}

impl Editor {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            cursor: 0,
            preferred_col: None,
            viewport_top: 0,
            viewport_left: 0,
            viewport_rows: 0,
            viewport_cols: 0,
        }
    }

    pub fn text(&self) -> &str {
        &self.buffer
    }

    pub fn cursor(&self) -> usize {
        self.cursor
    }

    pub fn len(&self) -> usize {
        self.buffer.len()
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn insert(&mut self, c: char) {
        self.buffer.insert(self.cursor, c);
        self.cursor += c.len_utf8();
        self.preferred_col = None;
    }

    pub fn insert_str(&mut self, s: &str) {
        self.buffer.insert_str(self.cursor, s);
        self.cursor += s.len();
        self.preferred_col = None;
    }

    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            let start = self.prev_boundary(self.cursor);
            self.buffer.remove(start);
            self.cursor = start;
            self.preferred_col = None;
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.buffer.len() {
            self.buffer.remove(self.cursor);
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

    pub fn set_viewport(&mut self, rows: usize, cols: usize) {
        self.viewport_rows = rows;
        self.viewport_cols = cols;
    }

    pub fn visible_lines(&self) -> impl Iterator<Item = &str> {
        let mut curr_start = 0;
        let mut curr_line = 0;

        std::iter::from_fn(move || {
            while curr_line < self.viewport_top && curr_start < self.buffer.len() {
                curr_start = self.line_end(curr_start) + 1;
                curr_line += 1;
            }

            while curr_line >= self.viewport_top
                && curr_line < self.viewport_top + self.viewport_rows
                && curr_start < self.buffer.len()
            {
                let curr_end = self.line_end(curr_start);

                let start = self.advance_chars(curr_start, self.viewport_left, curr_end);
                let end = self.advance_chars(start, self.viewport_cols, curr_end);

                let s = &self.buffer[start..end];

                curr_start = curr_end + 1;
                curr_line += 1;

                return Some(s);
            }

            None
        })
    }

    pub fn cursor_screen_pos(&self) -> Option<(usize, usize)> {
        let row = self.line_number(self.cursor);
        let col = self.char_column(self.cursor);

        let row_ok = row >= self.viewport_top && row < self.viewport_top + self.viewport_rows;
        let col_ok = col >= self.viewport_left && col < self.viewport_left + self.viewport_cols;

        (row_ok && col_ok).then(|| (row - self.viewport_top, col - self.viewport_left))
    }

    pub fn scroll_to_cursor(&mut self) {
        let row = self.line_number(self.cursor);
        let col = self.char_column(self.cursor);

        if row < self.viewport_top {
            self.viewport_top = row;
        } else if row >= self.viewport_top + self.viewport_rows {
            self.viewport_top = row - self.viewport_rows + 1;
        }

        if col < self.viewport_left {
            self.viewport_left = col;
        } else if col >= self.viewport_left + self.viewport_cols {
            self.viewport_left = col - self.viewport_cols + 1;
        }
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

    fn char_column(&self, offset: usize) -> usize {
        self.buffer[self.line_start(offset)..offset].chars().count()
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

    fn line_number(&self, offset: usize) -> usize {
        self.buffer[..offset]
            .bytes()
            .filter(|&b| b == b'\n')
            .count()
    }

    fn line_start(&self, offset: usize) -> usize {
        self.buffer[..offset].rfind('\n').map_or_default(|i| i + 1)
    }

    fn line_end(&self, offset: usize) -> usize {
        self.buffer[offset..]
            .find('\n')
            .map_or(self.buffer.len(), |i| i + offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ed(s: &str) -> Editor {
        Editor::from(s)
    }

    fn ed_vp(s: &str, top: usize, left: usize, rows: usize, cols: usize) -> Editor {
        let mut e = Editor::from(s);
        e.viewport_top = top;
        e.viewport_left = left;
        e.set_viewport(rows, cols);
        e
    }

    mod construction {
        use super::*;

        #[test]
        fn new_is_empty() {
            let e = Editor::new();
            assert_eq!(e.text(), "");
            assert_eq!(e.len(), 0);
            assert!(e.is_empty());
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn default_matches_new() {
            let a = Editor::default();
            let b = Editor::new();
            assert_eq!(a.text(), b.text());
            assert_eq!(a.cursor(), b.cursor());
            assert_eq!(a.viewport_rows, b.viewport_rows);
            assert_eq!(a.viewport_cols, b.viewport_cols);
        }

        #[test]
        fn from_str() {
            let e = Editor::from("hello");
            assert_eq!(e.text(), "hello");
            assert_eq!(e.len(), 5);
            assert!(!e.is_empty());
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn from_string() {
            let e = Editor::from(String::from("hello"));
            assert_eq!(e.text(), "hello");
            assert_eq!(e.len(), 5);
        }

        #[test]
        fn from_empty() {
            let e = Editor::from("");
            assert!(e.is_empty());
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn from_multibyte() {
            let e = Editor::from("héllo");
            assert_eq!(e.text(), "héllo");
            assert_eq!(e.len(), 6);
        }
    }

    mod insert {
        use super::*;

        #[test]
        fn char_into_empty() {
            let mut e = ed("");
            e.insert('h');
            e.insert('i');
            assert_eq!(e.text(), "hi");
            assert_eq!(e.cursor(), 2);
        }

        #[test]
        fn char_at_start() {
            let mut e = ed("world");
            e.insert('!');
            assert_eq!(e.text(), "!world");
            assert_eq!(e.cursor(), 1);
        }

        #[test]
        fn char_in_middle() {
            let mut e = ed("hllo");
            e.move_right();
            e.insert('e');
            assert_eq!(e.text(), "hello");
            assert_eq!(e.cursor(), 2);
        }

        #[test]
        fn char_at_end() {
            let mut e = ed("hell");
            e.move_end();
            e.insert('o');
            assert_eq!(e.text(), "hello");
            assert_eq!(e.cursor(), 5);
        }

        #[test]
        fn multibyte_char() {
            let mut e = ed("");
            e.insert('é');
            assert_eq!(e.text(), "é");
            assert_eq!(e.cursor(), 2);
        }

        #[test]
        fn char_before_multibyte() {
            let mut e = ed("é");
            e.insert('h');
            assert_eq!(e.text(), "hé");
            assert_eq!(e.cursor(), 1);
        }

        #[test]
        fn str_at_start() {
            let mut e = ed("world");
            e.insert_str("hello ");
            assert_eq!(e.text(), "hello world");
            assert_eq!(e.cursor(), 6);
        }

        #[test]
        fn str_multibyte() {
            let mut e = ed("");
            e.insert_str("héllo");
            assert_eq!(e.text(), "héllo");
            assert_eq!(e.cursor(), 6);
        }

        #[test]
        fn str_empty_is_noop() {
            let mut e = ed("hello");
            e.move_end();
            e.insert_str("");
            assert_eq!(e.text(), "hello");
            assert_eq!(e.cursor(), 5);
        }

        #[test]
        fn str_with_newline() {
            let mut e = ed("world");
            e.insert_str("hello\n");
            assert_eq!(e.text(), "hello\nworld");
            assert_eq!(e.cursor(), 6);
        }
    }

    mod delete {
        use super::*;

        #[test]
        fn back_at_start_is_noop() {
            let mut e = ed("hello");
            e.delete_back();
            assert_eq!(e.text(), "hello");
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn back_at_end() {
            let mut e = ed("hello");
            e.move_end();
            e.delete_back();
            assert_eq!(e.text(), "hell");
            assert_eq!(e.cursor(), 4);
        }

        #[test]
        fn back_from_middle() {
            let mut e = ed("hello");
            for _ in 0..3 {
                e.move_right();
            }
            e.delete_back();
            assert_eq!(e.text(), "helo");
            assert_eq!(e.cursor(), 2);
        }

        #[test]
        fn back_multibyte() {
            let mut e = ed("héllo");
            e.move_end();
            e.delete_back();
            e.delete_back();
            e.delete_back();
            e.delete_back();
            assert_eq!(e.text(), "h");
            assert_eq!(e.cursor(), 1);
        }

        #[test]
        fn back_to_empty() {
            let mut e = ed("abc");
            e.move_end();
            for _ in 0..3 {
                e.delete_back();
            }
            assert_eq!(e.text(), "");
            assert_eq!(e.cursor(), 0);
            e.delete_back();
            assert_eq!(e.text(), "");
        }

        #[test]
        fn back_across_newline_merges_lines() {
            let mut e = ed("ab\ncd");
            e.move_down();
            e.delete_back();
            assert_eq!(e.text(), "abcd");
            assert_eq!(e.cursor(), 2);
        }

        #[test]
        fn forward_at_end_is_noop() {
            let mut e = ed("hello");
            e.move_end();
            e.delete_forward();
            assert_eq!(e.text(), "hello");
            assert_eq!(e.cursor(), 5);
        }

        #[test]
        fn forward_at_start() {
            let mut e = ed("hello");
            e.delete_forward();
            assert_eq!(e.text(), "ello");
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn forward_multibyte() {
            let mut e = ed("héllo");
            e.move_right();
            e.delete_forward();
            assert_eq!(e.text(), "hllo");
            assert_eq!(e.cursor(), 1);
        }

        #[test]
        fn forward_repeated() {
            let mut e = ed("abc");
            e.delete_forward();
            e.delete_forward();
            assert_eq!(e.text(), "c");
        }

        #[test]
        fn forward_across_newline_keeps_cursor_position() {
            let mut e = ed("ab\ncd");
            e.move_end();
            e.delete_forward();
            assert_eq!(e.text(), "abcd");
            assert_eq!(e.cursor(), 2);
        }
    }

    mod movement_horizontal {
        use super::*;

        #[test]
        fn left_at_start_is_noop() {
            let mut e = ed("hi");
            e.move_left();
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn right_at_end_is_noop() {
            let mut e = ed("hi");
            e.move_end();
            e.move_right();
            assert_eq!(e.cursor(), 2);
        }

        #[test]
        fn across_ascii() {
            let mut e = ed("abc");
            for _ in 0..3 {
                e.move_right();
            }
            assert_eq!(e.cursor(), 3);
            for _ in 0..3 {
                e.move_left();
            }
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn across_multibyte() {
            let mut e = ed("héllo");
            e.move_right();
            assert_eq!(e.cursor(), 1);
            e.move_right();
            assert_eq!(e.cursor(), 3);
            e.move_left();
            assert_eq!(e.cursor(), 1);
        }

        #[test]
        fn home_to_line_start() {
            let mut e = ed("hello\nworld");
            e.move_down();
            e.move_end();
            e.move_home();
            assert_eq!(e.cursor(), 6);
        }

        #[test]
        fn home_at_line_start_is_noop() {
            let mut e = ed("hello");
            e.move_home();
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn end_to_line_end() {
            let mut e = ed("hello\nworld");
            e.move_end();
            assert_eq!(e.cursor(), 5);
        }

        #[test]
        fn end_on_last_line() {
            let mut e = ed("hello\nworld");
            e.move_down();
            e.move_end();
            assert_eq!(e.cursor(), 11);
        }

        #[test]
        fn end_does_not_cross_newline() {
            let mut e = ed("ab\ncd");
            e.move_end();
            assert_eq!(e.cursor(), 2);
            e.move_right();
            assert_eq!(e.cursor(), 3);
        }

        #[test]
        fn home_from_middle_of_line() {
            let mut e = ed("hello\nworld");
            e.move_down();
            for _ in 0..3 {
                e.move_right();
            }
            assert_eq!(e.cursor(), 9);
            e.move_home();
            assert_eq!(e.cursor(), 6);
        }
    }

    mod movement_vertical {
        use super::*;

        #[test]
        fn up_at_first_line_is_noop() {
            let mut e = ed("hello\nworld");
            e.move_up();
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn down_at_last_line_is_noop() {
            let mut e = ed("hello");
            e.move_down();
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn down_preserves_column() {
            let mut e = ed("aaaa\nbbbb\ncccc");
            e.move_right();
            e.move_right();
            assert_eq!(e.cursor(), 2);
            e.move_down();
            assert_eq!(e.cursor(), 7);
            e.move_down();
            assert_eq!(e.cursor(), 12);
        }

        #[test]
        fn up_preserves_column() {
            let mut e = ed("aaaa\nbbbb\ncccc");
            e.move_down();
            e.move_down();
            for _ in 0..3 {
                e.move_right();
            }
            assert_eq!(e.cursor(), 13);
            e.move_up();
            assert_eq!(e.cursor(), 8);
            e.move_up();
            assert_eq!(e.cursor(), 3);
        }

        #[test]
        fn down_clamps_to_shorter_line() {
            let mut e = ed("aaaaaaaa\nbb");
            e.move_end();
            assert_eq!(e.cursor(), 8);
            e.move_down();
            assert_eq!(e.cursor(), 11);
        }

        #[test]
        fn up_clamps_to_shorter_line() {
            let mut e = ed("bb\naaaaaaaa");
            e.move_down();
            e.move_end();
            assert_eq!(e.cursor(), 11);
            e.move_up();
            assert_eq!(e.cursor(), 2);
        }

        #[test]
        fn with_multibyte() {
            let mut e = ed("αβγδ\naaaa");
            e.move_right();
            e.move_right();
            assert_eq!(e.cursor(), 4);
            e.move_down();
            assert_eq!(e.cursor(), 11);
        }

        #[test]
        fn on_empty_is_noop() {
            let mut e = ed("");
            e.move_down();
            assert_eq!(e.cursor(), 0);
            e.move_up();
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn through_many_lines() {
            let mut e = ed("a\nb\nc\nd\ne");
            for _ in 0..4 {
                e.move_down();
            }
            assert_eq!(e.cursor(), 8);
            for _ in 0..4 {
                e.move_up();
            }
            assert_eq!(e.cursor(), 0);
        }
    }

    mod preferred_col {
        use super::*;

        #[test]
        fn restored_on_subsequent_vertical_moves() {
            let mut e = ed("0123456789\nabcde\nfghijk");
            e.move_down();
            e.move_down();
            for _ in 0..6 {
                e.move_right();
            }
            assert_eq!(e.cursor(), 23);

            e.move_up();
            assert_eq!(e.cursor(), 16);

            e.move_up();
            assert_eq!(e.cursor(), 6);
        }

        #[test]
        fn horizontal_move_clears() {
            let mut e = ed("0123456789\nabcde");
            e.move_end();
            e.move_down();
            assert_eq!(e.cursor(), 16);

            e.move_left();
            assert_eq!(e.cursor(), 15);

            e.move_up();
            assert_eq!(e.cursor(), 4);
        }

        #[test]
        fn home_clears() {
            let mut e = ed("0123456789\nabcde\nfghijk");
            e.move_end();
            e.move_down();
            e.move_down();
            assert_eq!(e.cursor(), 23);

            e.move_home();
            e.move_up();
            assert_eq!(e.cursor(), 11);
        }

        #[test]
        fn end_clears() {
            let mut e = ed("0123456789\nabc");
            e.move_end();
            e.move_down();
            assert_eq!(e.cursor(), 14);

            e.move_end();
            e.move_up();
            assert_eq!(e.cursor(), 3);
        }

        #[test]
        fn noop_up_at_first_line_does_not_poison() {
            let mut e = ed("aaaaaaaa\nbb");
            e.move_up();
            assert_eq!(e.cursor(), 0);
            e.move_down();
            assert_eq!(e.cursor(), 9);
        }

        #[test]
        fn insert_clears() {
            let mut e = ed("0123456789\nabcdefghij");
            for _ in 0..3 {
                e.move_right();
            }
            e.move_down();
            assert_eq!(e.cursor(), 14);

            e.insert('X');
            e.move_up();
            assert_eq!(e.cursor(), 4);
        }

        #[test]
        fn delete_back_clears() {
            let mut e = ed("0123456789\nabcdefghij");
            e.move_end();
            e.move_down();
            assert_eq!(e.cursor(), 21);
            for _ in 0..5 {
                e.move_right();
            }
            assert_eq!(e.cursor(), 21);

            e.delete_back();
            assert_eq!(e.cursor(), 20);
            e.move_up();
            assert_eq!(e.cursor(), 9);
        }

        #[test]
        fn delete_forward_does_not_clear() {
            let mut e = ed("0123456789\nabcdefghij");
            for _ in 0..5 {
                e.move_right();
            }
            e.move_down();
            assert_eq!(e.cursor(), 16);

            e.delete_forward();
            assert_eq!(e.cursor(), 16);
            assert_eq!(e.text(), "0123456789\nabcdeghij");

            e.move_up();
            assert_eq!(e.cursor(), 5);
        }
    }

    mod viewport {
        use super::*;

        #[test]
        fn visible_lines_no_scroll() {
            let e = ed_vp("line1\nline2\nline3", 0, 0, 10, 80);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert_eq!(lines, vec!["line1", "line2", "line3"]);
        }

        #[test]
        fn visible_lines_respects_row_count() {
            let e = ed_vp("l1\nl2\nl3\nl4\nl5", 0, 0, 3, 80);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert_eq!(lines, vec!["l1", "l2", "l3"]);
        }

        #[test]
        fn visible_lines_with_vertical_scroll() {
            let e = ed_vp("l1\nl2\nl3\nl4\nl5", 2, 0, 2, 80);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert_eq!(lines, vec!["l3", "l4"]);
        }

        #[test]
        fn visible_lines_with_horizontal_scroll() {
            let e = ed_vp("0123456789\nabcdefghij", 0, 3, 10, 4);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert_eq!(lines, vec!["3456", "defg"]);
        }

        #[test]
        fn visible_lines_horizontal_past_end() {
            let e = ed_vp("short\nline", 0, 20, 10, 40);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert_eq!(lines, vec!["", ""]);
        }

        #[test]
        fn visible_lines_empty_buffer() {
            let e = ed_vp("", 0, 0, 10, 80);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert!(lines.is_empty());
        }

        #[test]
        fn visible_lines_scroll_past_end() {
            let e = ed_vp("only\none", 100, 0, 5, 80);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert!(lines.is_empty());
        }

        #[test]
        fn visible_lines_multibyte() {
            let e = ed_vp("αβγδε\nζηθ", 0, 0, 10, 3);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert_eq!(lines, vec!["αβγ", "ζηθ"]);
        }

        #[test]
        fn visible_lines_multibyte_scrolled() {
            let e = ed_vp("αβγδε", 0, 2, 10, 2);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert_eq!(lines, vec!["γδ"]);
        }

        #[test]
        fn visible_lines_trailing_newline_with_scroll() {
            let e = ed_vp("a\nb\nc\n", 1, 0, 10, 80);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert_eq!(lines, vec!["b", "c"]);
        }

        #[test]
        fn visible_lines_scroll_exactly_to_last_line() {
            let e = ed_vp("a\nb\nc", 3, 0, 5, 80);
            let lines: Vec<&str> = e.visible_lines().collect();
            assert!(lines.is_empty());
        }

        #[test]
        fn cursor_at_origin() {
            let e = ed_vp("hello", 0, 0, 10, 80);
            assert_eq!(e.cursor_screen_pos(), Some((0, 0)));
        }

        #[test]
        fn cursor_after_move() {
            let mut e = ed_vp("hello\nworld", 0, 0, 10, 80);
            e.move_down();
            e.move_right();
            e.move_right();
            assert_eq!(e.cursor_screen_pos(), Some((1, 2)));
        }

        #[test]
        fn cursor_none_off_screen_above() {
            let e = ed_vp("l1\nl2\nl3", 1, 0, 10, 80);
            assert_eq!(e.cursor_screen_pos(), None);
        }

        #[test]
        fn cursor_none_off_screen_below() {
            let mut e = ed_vp("l1\nl2\nl3\nl4\nl5", 0, 0, 2, 80);
            for _ in 0..3 {
                e.move_down();
            }
            assert_eq!(e.cursor_screen_pos(), None);
        }

        #[test]
        fn cursor_none_when_row_is_first_off_screen() {
            let mut e = ed_vp("l1\nl2\nl3\nl4", 0, 0, 3, 80);
            for _ in 0..3 {
                e.move_down();
            }
            assert_eq!(e.cursor_screen_pos(), None);
        }

        #[test]
        fn cursor_none_horizontally_off() {
            let e = ed_vp("0123456789", 0, 20, 10, 4);
            assert_eq!(e.cursor_screen_pos(), None);
        }

        #[test]
        fn cursor_none_when_col_is_first_off_screen() {
            let mut e = ed_vp("0123456789", 0, 0, 10, 3);
            for _ in 0..3 {
                e.move_right();
            }
            assert_eq!(e.cursor_screen_pos(), None);
        }

        #[test]
        fn cursor_screen_pos_accounts_for_horizontal_scroll() {
            let mut e = ed_vp("0123456789", 0, 3, 10, 5);
            for _ in 0..4 {
                e.move_right();
            }
            assert_eq!(e.cursor_screen_pos(), Some((0, 1)));
        }

        #[test]
        fn cursor_screen_pos_accounts_for_vertical_scroll() {
            let mut e = ed_vp("l1\nl2\nl3\nl4\nl5", 2, 0, 5, 80);
            for _ in 0..3 {
                e.move_down();
            }
            assert_eq!(e.cursor_screen_pos(), Some((1, 0)));
        }

        #[test]
        fn scroll_down_when_cursor_below() {
            let mut e = ed_vp("l1\nl2\nl3\nl4\nl5", 0, 0, 2, 80);
            for _ in 0..4 {
                e.move_down();
            }
            e.scroll_to_cursor();
            assert_eq!(e.viewport_top, 3);
        }

        #[test]
        fn scroll_down_uses_correct_arithmetic() {
            let text = "l1\nl2\nl3\nl4\nl5\nl6\nl7\nl8\nl9\nl10\nl11";
            let mut e = ed_vp(text, 0, 0, 5, 80);
            for _ in 0..10 {
                e.move_down();
            }
            e.scroll_to_cursor();
            assert_eq!(e.viewport_top, 6);
        }

        #[test]
        fn scroll_up_when_cursor_above() {
            let mut e = ed_vp("l1\nl2\nl3\nl4\nl5", 3, 0, 2, 80);
            e.scroll_to_cursor();
            assert_eq!(e.viewport_top, 0);
        }

        #[test]
        fn scroll_no_change_when_visible() {
            let mut e = ed_vp("l1\nl2\nl3", 0, 0, 10, 80);
            e.move_down();
            e.scroll_to_cursor();
            assert_eq!(e.viewport_top, 0);
        }

        #[test]
        fn scroll_horizontal() {
            let mut e = ed_vp("0123456789", 0, 0, 10, 4);
            for _ in 0..6 {
                e.move_right();
            }
            e.scroll_to_cursor();
            assert_eq!(e.viewport_left, 3);
        }

        #[test]
        fn scroll_left_when_cursor_left_of_viewport() {
            let mut e = ed_vp("0123456789", 0, 5, 10, 4);
            e.scroll_to_cursor();
            assert_eq!(e.viewport_left, 0);
        }

        #[test]
        fn scroll_to_row_boundary_with_zero_rows() {
            let mut e = ed_vp("l1\nl2\nl3", 1, 0, 0, 80);
            e.move_down();
            e.scroll_to_cursor();
            assert_eq!(e.viewport_top, 2);
        }

        #[test]
        fn scroll_to_col_boundary_with_zero_cols() {
            let mut e = ed_vp("hello", 0, 1, 10, 0);
            e.move_right();
            e.scroll_to_cursor();
            assert_eq!(e.viewport_left, 2);
        }

        #[test]
        fn cursor_visible_after_scroll() {
            let mut e = ed_vp("l1\nl2\nl3\nl4\nl5", 0, 0, 2, 80);
            for _ in 0..4 {
                e.move_down();
            }
            e.scroll_to_cursor();
            assert_eq!(e.cursor_screen_pos(), Some((1, 0)));
        }
    }

    mod properties {
        use super::*;

        #[test]
        fn right_then_left_is_identity() {
            let s = "héllo wörld";
            for (i, _) in s.char_indices() {
                let mut e = ed(s);
                e.cursor = i;
                e.move_right();
                e.move_left();
                assert_eq!(e.cursor(), i);
            }
        }

        #[test]
        fn left_then_right_is_identity() {
            let s = "héllo wörld";
            for (i, _) in s.char_indices() {
                if i == 0 {
                    continue;
                }
                let mut e = ed(s);
                e.cursor = i;
                e.move_left();
                e.move_right();
                assert_eq!(e.cursor(), i);
            }
        }

        #[test]
        fn insert_then_delete_restores_text() {
            let mut e = ed("hello");
            e.move_end();
            e.insert_str(" world");
            assert_eq!(e.text(), "hello world");
            for _ in 0..6 {
                e.delete_back();
            }
            assert_eq!(e.text(), "hello");
            assert_eq!(e.cursor(), 5);
        }

        #[test]
        fn forward_delete_then_insert_restores() {
            let mut e = ed("hello");
            e.move_end();
            e.move_left();
            let c = e.text().chars().nth(4).unwrap();
            e.delete_forward();
            assert_eq!(e.text(), "hell");
            e.insert(c);
            assert_eq!(e.text(), "hello");
        }

        #[test]
        fn cursor_always_on_char_boundary_after_moves() {
            let s = "αβγδε";
            let mut e = ed(s);
            for _ in 0..10 {
                e.move_right();
            }
            assert!(s.is_char_boundary(e.cursor()));
            for _ in 0..10 {
                e.move_left();
            }
            assert!(s.is_char_boundary(e.cursor()));
        }
    }

    mod stress {
        use super::*;

        #[test]
        fn type_sentence_character_by_character() {
            let target = "the quick brown fox jumps over the lazy dog";
            let mut e = ed("");
            for c in target.chars() {
                e.insert(c);
            }
            assert_eq!(e.text(), target);
            assert_eq!(e.cursor(), target.len());
        }

        #[test]
        fn undo_typing_by_backspacing() {
            let target = "hello world";
            let mut e = ed("");
            for c in target.chars() {
                e.insert(c);
            }
            for _ in 0..target.chars().count() {
                e.delete_back();
            }
            assert_eq!(e.text(), "");
            assert_eq!(e.cursor(), 0);
        }

        #[test]
        fn edit_middle_of_long_line() {
            let mut e = ed("the quick brown fox");
            for _ in 0..4 {
                e.move_right();
            }
            e.insert_str("very ");
            assert_eq!(e.text(), "the very quick brown fox");
        }

        #[test]
        fn multiline_navigation_and_edit() {
            let mut e = ed("one\ntwo\nthree\nfour\nfive");
            e.move_down();
            e.move_down();
            e.move_end();
            e.insert_str("!");
            assert_eq!(e.text(), "one\ntwo\nthree!\nfour\nfive");
        }

        #[test]
        fn alternating_inserts_and_moves() {
            let mut e = ed("");
            e.insert('a');
            e.insert('c');
            e.move_left();
            e.insert('b');
            assert_eq!(e.text(), "abc");
            assert_eq!(e.cursor(), 2);
        }
    }
}
