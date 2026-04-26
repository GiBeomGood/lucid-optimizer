use std::io;
use std::time::Duration;

use crossterm::{
    event::{self as ct_event, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};

mod app;
mod event;
mod item;
mod stats;
mod storage;
mod ui;

use app::App;
use event::key_to_action;

fn main() -> io::Result<()> {
    let path = std::env::args().nth(1).unwrap_or_else(|| "items.json".to_string());
    let stats_path = std::env::args().nth(2).unwrap_or_else(|| "stats.json".to_string());
    let items = storage::load(&path).unwrap_or_default();
    let stats = storage::load_stats(&stats_path).unwrap_or_default();
    let mut app = App::new(items, path, stats, stats_path);

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture);
        original_hook(info);
    }));

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if ct_event::poll(Duration::from_millis(100))?
            && let Event::Key(key) = ct_event::read()?
        {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if let Some(action) = key_to_action(&app, key) {
                app.apply(action);
            }
        }

        app.tick();

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}
