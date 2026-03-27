use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};

mod models;
mod app;
mod ui;

use crate::models::{App, Screen};

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let tick_rate = Duration::from_millis(33);
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app, tick_rate);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
    tick_rate: Duration,
) -> Result<(), Box<dyn Error>> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| ui::ui(f, app)).map_err(|e| e.to_string())?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.screen {
                    Screen::Menu => {
                        match key.code {
                            KeyCode::Enter => app.init_level(),
                            KeyCode::Char('u') | KeyCode::Char('t') => app.screen = Screen::Upgrades,
                            _ => {}
                        }
                    }
                    Screen::Gameplay => {
                        match key.code {
                            KeyCode::Left => { if app.cannon_x > 5 { app.cannon_x -= 1; } }
                            KeyCode::Right => { if app.cannon_x < 35 { app.cannon_x += 1; } }
                            KeyCode::Char('q') => app.screen = Screen::Menu,
                            _ => {}
                        }
                    }
                    Screen::Upgrades => {
                        match key.code {
                            KeyCode::Char('b') => app.screen = Screen::Menu,
                            KeyCode::Char('1') => {
                                let cost = 100 + (app.fire_rate as u32 * 10);
                                if app.bits >= cost {
                                    app.bits -= cost;
                                    app.fire_rate += 5;
                                }
                            }
                            KeyCode::Char('2') => {
                                let cost = app.mob_packet_size * 500;
                                if app.bits >= cost {
                                    app.bits -= cost;
                                    app.mob_packet_size += 1;
                                }
                            }
                            KeyCode::Char('3') => {
                                let cost = 300 + (app.mob_speed * 1000.0) as u32;
                                if app.bits >= cost {
                                    app.bits -= cost;
                                    app.mob_speed += 0.1;
                                }
                            }
                            KeyCode::Char('4') => {
                                let cost = app.mob_damage as u32 * 1000;
                                if app.bits >= cost {
                                    app.bits -= cost;
                                    app.mob_damage += 1;
                                }
                            }
                            _ => {}
                        }
                    }
                    Screen::LevelComplete => {
                        if key.code == KeyCode::Enter {
                            app.level += 1;
                            app.screen = Screen::Menu;
                        }
                    }
                    Screen::GameOver => {
                        if key.code == KeyCode::Enter {
                            app.level = 1;
                            app.bits = 0;
                            app.screen = Screen::Menu;
                        }
                    }
                }
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}
