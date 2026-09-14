use std::{
    borrow::Cow,
    fs::File,
    io::{self, Read, Write},
    ops::ControlFlow,
    path::{Path, PathBuf},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::Print,
    terminal::{
        self, BeginSynchronizedUpdate, EndSynchronizedUpdate, EnterAlternateScreen,
        LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};

use crate::{
    editor::Editor,
    message::{Message, MessageKind},
    prompt::{Prompt, PromptKind},
};

const GUTTER_MARGIN: usize = 3;

#[derive(Debug, Default)]
struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> io::Result<Self> {
        enable_raw_mode()?;
        execute!(io::stdout(), EnterAlternateScreen)?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
    }
}

#[derive(Debug, PartialEq)]
enum LineEnding {
    LF,
    CRLF,
}

impl Default for LineEnding {
    fn default() -> Self {
        LineEnding::LF
    }
}

#[derive(Debug, Default)]
pub struct App {
    path: Option<PathBuf>,
    editor: Editor,
    line_ending: LineEnding,
    prompt: Option<Prompt>,
    message: Option<Message>,
    terminal_rows: usize,
    terminal_cols: usize,
}

impl App {
    pub fn new(path: Option<&Path>) -> Self {
        match path {
            Some(p) => Self::load(p).unwrap_or_else(|e| {
                let mut a = Self::default();
                a.path = Some(p.to_path_buf());
                a.message = Some(Message::new(&e.to_string(), MessageKind::FailedOpen));
                a
            }),
            None => Self::default(),
        }
    }

    fn set_prompt(&mut self, prompt: Prompt) {
        self.prompt = Some(prompt);
    }

    fn set_message(&mut self, message: Message) {
        self.message = Some(message);
    }

    fn remove_prompt(&mut self) {
        self.prompt = None;
    }

    fn remove_message(&mut self) {
        self.message = None;
    }

    pub fn run(&mut self) -> io::Result<()> {
        let _guard = TerminalGuard::enter()?;
        let (cols, rows) = terminal::size()?;
        self.terminal_rows = rows as usize;
        self.terminal_cols = cols as usize;

        self.event_loop()?;

        Ok(())
    }

    fn load(path: &Path) -> io::Result<Self> {
        let mut text = String::new();
        File::open(path)?.read_to_string(&mut text)?;

        let line_ending = match text.rfind('\n') {
            Some(i) if i > 0 && text.as_bytes()[i - 1] == b'\r' => LineEnding::CRLF,
            _ => LineEnding::LF,
        };

        if line_ending == LineEnding::CRLF {
            text = text.replace("\r\n", "\n");
        }

        Ok(Self {
            path: Some(path.to_owned()),
            editor: Editor::from(text),
            line_ending,
            prompt: None,
            message: None,
            terminal_rows: 0,
            terminal_cols: 0,
        })
    }

    fn write_to(&self, path: &Path) -> io::Result<()> {
        let text = self.editor.text();
        let str = match self.line_ending {
            LineEnding::LF => Cow::Borrowed(text.as_str()),
            LineEnding::CRLF => Cow::Owned(text.replace('\n', "\r\n")),
        };
        File::create(path)?.write_all(str.as_bytes())
    }

    fn save(&self) -> Option<io::Result<()>> {
        self.path.as_ref().map(|p| self.write_to(p))
    }

    fn save_as(&mut self, path: &Path) -> io::Result<()> {
        self.write_to(path)?;
        self.path = Some(path.to_owned());
        Ok(())
    }

    fn event_loop(&mut self) -> io::Result<()> {
        loop {
            self.update_viewport();
            self.draw()?;

            let event = event::read()?;
            if self.handle_event(event).is_break() {
                execute!(io::stdout(), cursor::Show)?;
                break;
            }

            if let Some(ref mut prompt) = self.prompt {
                prompt.scroll_to_cursor();
            } else {
                self.editor.scroll_to_cursor();
            }
        }

        Ok(())
    }

    fn update_viewport(&mut self) {
        let gutter_width = self.editor.line_count().ilog10() as usize + 1;
        let margin = gutter_width + GUTTER_MARGIN;
        let content_cols = self.terminal_cols.saturating_sub(margin);
        let content_rows = self.terminal_rows.saturating_sub(1);
        self.editor.set_viewport_size(content_cols, content_rows);

        if let Some(ref mut p) = self.prompt {
            p.set_viewport_size(content_cols);
        }
    }

    fn draw(&self) -> io::Result<()> {
        let mut out = io::stdout();

        queue!(out, BeginSynchronizedUpdate)?;
        self.draw_editor(&mut out)?;
        self.draw_status(&mut out)?;
        self.draw_cursor(&mut out)?;
        queue!(out, EndSynchronizedUpdate)?;

        out.flush()?;

        Ok(())
    }

    fn draw_editor(&self, out: &mut io::Stdout) -> io::Result<()> {
        let line_count = self.editor.line_count();
        let gutter_width = line_count.ilog10() as usize + 1;
        let (_, v_top, _, _) = self.editor.viewport();

        queue!(
            out,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, 0),
        )?;

        for (i, line) in self.editor.visible_lines().enumerate() {
            let num = v_top + i + 1;
            let line_out = format!("{:>width$} | {}\r\n", num, line, width = gutter_width);
            queue!(out, Print(line_out))?;
        }

        Ok(())
    }

    fn draw_status(&self, out: &mut io::Stdout) -> io::Result<()> {
        let last_row = self.terminal_rows.saturating_sub(1) as u16;

        let mut line = if let Some(ref p) = self.prompt {
            p.visible_line()
        } else if let Some(ref m) = self.message {
            m.text()
        } else {
            self.default_status()
        };

        line.truncate(self.terminal_cols);
        queue!(out, cursor::MoveTo(0, last_row), Print(line))?;
        Ok(())
    }

    fn draw_cursor(&self, out: &mut io::Stdout) -> io::Result<()> {
        if let Some(p) = &self.prompt {
            let col = p.cursor_pos();
            let (v_left, _) = p.viewport();
            let last_row = self.terminal_rows.saturating_sub(1) as u16;
            queue!(
                out,
                cursor::Show,
                cursor::MoveTo((col - v_left) as u16, last_row)
            )?;
            return Ok(());
        }

        let (col, row) = self.editor.cursor_pos();
        let (v_left, v_top, v_cols, v_rows) = self.editor.viewport();
        let visible =
            col >= v_left && col < v_left + v_cols && row >= v_top && row < v_top + v_rows;
        if visible {
            let gutter_width = self.editor.line_count().ilog10() as usize + 1;
            let margin = gutter_width + GUTTER_MARGIN;
            queue!(
                out,
                cursor::Show,
                cursor::MoveTo((col - v_left + margin) as u16, (row - v_top) as u16,)
            )?;
        } else {
            queue!(out, cursor::Hide)?;
        }
        Ok(())
    }

    fn default_status(&self) -> String {
        let (col, row) = self.editor.cursor_pos();
        let name = self
            .path
            .as_deref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "new".into());
        let pos = format!("{}:{}", row + 1, col + 1);

        let pos_len = pos.chars().count();
        let name_len = name.chars().count();

        let name = if name_len + pos_len > self.terminal_cols {
            let keep = self.terminal_cols.saturating_sub(pos_len);
            let skip = name_len.saturating_sub(keep);
            name.chars().skip(skip).collect::<String>()
        } else {
            name
        };

        let name_len = name.chars().count();
        let padding = self.terminal_cols.saturating_sub(name_len + pos_len);
        format!("{name}{}{pos}", " ".repeat(padding))
    }

    fn handle_event(&mut self, event: Event) -> ControlFlow<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_keys(key),

            Event::Resize(cols, rows) => {
                self.terminal_rows = rows as usize;
                self.terminal_cols = cols as usize;
                ControlFlow::Continue(())
            }

            _ => ControlFlow::Continue(()),
        }
    }

    fn handle_keys(&mut self, key: KeyEvent) -> ControlFlow<()> {
        if self.prompt.is_some() {
            self.handle_prompt_keys(key)
        } else {
            self.handle_editor_keys(key)
        }
    }

    fn handle_editor_keys(&mut self, key: KeyEvent) -> ControlFlow<()> {
        self.remove_message();

        match key.code {
            KeyCode::Char('o') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.set_prompt(Prompt::new(PromptKind::Open));
            }

            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                let result = self.save();
                match result {
                    Some(Ok(())) => {
                        let path_str = &self
                            .path
                            .as_ref()
                            .expect("Some(Ok(())) ensures that self.path is Some")
                            .to_string_lossy()
                            .to_string();

                        self.set_message(Message::new(path_str, MessageKind::Saved));
                    }
                    Some(Err(e)) => {
                        self.set_message(Message::new(&e.to_string(), MessageKind::FailedSave));
                    }
                    None => self.set_prompt(Prompt::new(PromptKind::SaveAs)),
                }
            }

            KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.delete_line()
            }

            KeyCode::F(2) => {
                self.set_prompt(Prompt::new(PromptKind::SaveAs));
            }

            KeyCode::Esc => return ControlFlow::Break(()),

            KeyCode::Enter => self.editor.insert('\n'),
            KeyCode::Tab => self.editor.insert_str("    "),

            KeyCode::Backspace => self.editor.delete_back(),
            KeyCode::Delete => self.editor.delete_forward(),

            KeyCode::Char(c) => self.editor.insert(c),

            KeyCode::Home if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.move_home_full()
            }

            KeyCode::End if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.move_end_full()
            }

            KeyCode::Home => self.editor.move_home(),
            KeyCode::End => self.editor.move_end(),

            KeyCode::Up => self.editor.move_up(),
            KeyCode::Down => self.editor.move_down(),

            KeyCode::Left if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.move_word_left()
            }

            KeyCode::Right if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.editor.move_word_right()
            }

            KeyCode::Left => self.editor.move_left(),
            KeyCode::Right => self.editor.move_right(),

            _ => {}
        }

        ControlFlow::Continue(())
    }

    fn handle_prompt_keys(&mut self, key: KeyEvent) -> ControlFlow<()> {
        let prompt = self
            .prompt
            .as_mut()
            .expect("handle_keys ensures that prompt is Some");

        match key.code {
            KeyCode::Esc => self.remove_prompt(),

            KeyCode::Enter => self.submit_prompt(),

            KeyCode::Backspace => prompt.delete_back(),
            KeyCode::Delete => prompt.delete_forward(),

            KeyCode::Char(c) => prompt.insert(c),

            KeyCode::Home => prompt.move_home(),
            KeyCode::End => prompt.move_end(),
            KeyCode::Left => prompt.move_left(),
            KeyCode::Right => prompt.move_right(),

            _ => {}
        }

        ControlFlow::Continue(())
    }

    fn submit_prompt(&mut self) {
        let prompt = self
            .prompt
            .take()
            .expect("handle_keys ensures that prompt is Some");

        let prompt_str = prompt.prompt();
        match prompt.kind() {
            PromptKind::Open => {
                let path = PathBuf::from(prompt_str);
                let new = Self::new(Some(&path));
                self.path = new.path;
                self.editor = new.editor;
                self.line_ending = new.line_ending;
                self.message = new.message;
            }
            PromptKind::SaveAs => {
                let path = PathBuf::from(prompt_str);
                let result = self.save_as(&path);

                match result {
                    Ok(()) => self.set_message(Message::new(prompt_str, MessageKind::Saved)),
                    Err(e) => {
                        self.set_message(Message::new(&e.to_string(), MessageKind::FailedSave));
                    }
                }
            }
        }
    }
}
