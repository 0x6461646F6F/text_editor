use crate::editor::Editor;
use crossterm::{cursor, queue, style::Print, terminal};
use std::io::{self, Write};

pub fn draw(editor: &Editor) -> io::Result<()> {
    let mut out = io::stdout();

    queue!(
        out,
        terminal::Clear(terminal::ClearType::All),
        cursor::MoveTo(0, 0),
    )?;

    for line in editor.visible_lines() {
        queue!(out, Print(line), Print("\r\n"))?;
    }

    if let Some((row, col)) = editor.cursor_screen_pos() {
        queue!(out, cursor::MoveTo(col as u16, row as u16))?;
    }

    out.flush()?;
    Ok(())
}
