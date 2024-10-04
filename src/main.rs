//! New york times games CLI client.
//! Supports Wordle, Connections and Strands so far

use std::error::Error;

use app::App;
use ratatui::{
    backend::CrosstermBackend,
    crossterm::{
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    Terminal,
};

mod app; // The application UI
mod state; // Load/saves the state
mod tabs; // Tabs for my game selection method

fn main() -> Result<(), Box<dyn Error>> {
    let mut app = App::default();

    // we must load the state. If there is any errors loading the state, we ignore them and use the
    // default state

    let state_loc = state::get_loc();
    let data = std::fs::read_to_string(&state_loc);

    if let Ok(d) = data {
        let a = state::load(d);

        if let Ok(a) = a {
            app = a;
        }
    }

    if app.current_game.0 == 255 {
        // no game currently

        if let Ok(d) = app.download() {
            app.current_game = d;
        }

        app.generate_game_string();
    }

    // loaded the state

    enable_raw_mode()?;

    execute!(std::io::stdout(), EnterAlternateScreen, EnableMouseCapture)?;

    let backend = CrosstermBackend::new(std::io::stdout());
    let mut terminal = Terminal::new(backend)?;

    // Run the app

    loop {
        terminal.draw(|frame| app::draw(frame, &mut app))?;

        if event::poll(std::time::Duration::from_secs_f32(0.05))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Left => {
                            app.left();
                        }

                        KeyCode::Right => app.right(),

                        KeyCode::Up => app.up(),

                        KeyCode::Down => app.down(),

                        KeyCode::Char('~') => {
                            app.quit();
                        }

                        KeyCode::Char(char) => {
                            app.key(char);
                        }

                        KeyCode::Backspace => {
                            if !app.guess_buffer.is_empty() {
                                app.guess_buffer.remove(app.guess_buffer.len() - 1);
                            }
                        }

                        KeyCode::Enter => {
                            app.enter();
                        }

                        _ => (),
                    }
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    // here we save the state

    app.should_quit = false;

    let data = state::save(app);

    if let Ok(d) = data {
        let _ = std::fs::write(state_loc, d);
    }

    Ok(())
}
