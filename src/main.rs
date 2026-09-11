mod editor;
mod render;
mod terminal;

use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal as term,
};

use editor::Editor;
use terminal::Terminal;

fn main() -> io::Result<()> {
    let _term = Terminal::enter()?;

    let (cols, rows) = term::size()?;
    let mut editor = Editor::from("hello\nworld\nthis is a test");
    editor.set_viewport((rows as usize).saturating_sub(1), cols as usize);

    run(&mut editor)
}

fn run(editor: &mut Editor) -> io::Result<()> {
    loop {
        render::draw(editor)?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match key.code {
                KeyCode::Esc => break,
                KeyCode::Char(c) => editor.insert(c),
                KeyCode::Enter => editor.insert('\n'),
                KeyCode::Tab => editor.insert_str("    "),
                KeyCode::Backspace => editor.delete_back(),
                KeyCode::Delete => editor.delete_forward(),
                KeyCode::Left => editor.move_left(),
                KeyCode::Right => editor.move_right(),
                KeyCode::Up => editor.move_up(),
                KeyCode::Down => editor.move_down(),
                KeyCode::Home => editor.move_home(),
                KeyCode::End => editor.move_end(),
                _ => {}
            }

            editor.scroll_to_cursor();
        }
    }
    Ok(())
}
