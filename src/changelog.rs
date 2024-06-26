use std::io::{stdout, Write};

use termimad::{Area, MadSkin, MadView};
use termimad::crossterm::{event, queue, terminal};
use termimad::crossterm::cursor::{Hide, Show};
use termimad::crossterm::event::{Event, KeyEvent};
use termimad::crossterm::terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen};

use crate::changelog;
use crate::result::PhotosExportResult;

const CHANGELOG: &str = include_str!("../CHANGELOG.md");

fn get_view_area() -> Area {
    Area::full_screen()
}

pub fn print_changelog() -> PhotosExportResult<()> {
    let mut w = stdout(); // we could also have used stderr

    queue!(w, EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    queue!(w, Hide)?; // hiding the cursor

    let mut view = MadView::from(changelog::CHANGELOG.to_owned(), get_view_area(), MadSkin::default());

    loop {
        view.write_on(&mut w)?;
        w.flush()?;
        match event::read() {
            Ok(Event::Key(KeyEvent{code, ..})) => {
                match code {
                    event::KeyCode::Up => view.try_scroll_lines(-1),
                    event::KeyCode::Down => view.try_scroll_lines(1),
                    event::KeyCode::PageUp => view.try_scroll_pages(-1),
                    event::KeyCode::PageDown => view.try_scroll_pages(1),
                    _ => break,
                }
            }
            Ok(Event::Resize(..)) => {
                queue!(w, Clear(ClearType::All))?;
                view.resize(&get_view_area());
            }
            _ => {}
        }
    }

    terminal::disable_raw_mode()?;
    queue!(w, Show)?; // we must restore the cursor
    queue!(w, LeaveAlternateScreen)?;

    w.flush()?;

    Ok(())
}