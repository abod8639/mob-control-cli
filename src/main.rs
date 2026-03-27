use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame, Terminal,
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use rand::RngExt; 

// --- Data Structures ---

#[derive(PartialEq)]
enum Screen {
    Menu,
    Gameplay,
    Upgrades,
    GameOver,
}

struct Mob {
    x: f64,
    y: f64,
    is_enemy: bool,
}

struct Gate {
    x: u16,
    y: u16,
    width: u16,
    multiplier: u32,
    is_add: bool,
}

struct App {
    screen: Screen,
    bits: u32,
    level: u32,
    cannon_x: u16,
    mobs: Vec<Mob>,
    gates: Vec<Gate>,
    enemy_base_hp: i32,
    fire_rate: u32,
    last_fire: Instant,
    last_tick: Instant,
}

impl App {
    fn new() -> App {
        App {
            screen: Screen::Menu,
            bits: 0,
            level: 4,
            cannon_x: 20,
            mobs: Vec::new(),
            gates: vec![
                Gate { x: 10, y: 15, width: 10, multiplier: 2, is_add: false },
                Gate { x: 25, y: 10, width: 8, multiplier: 15, is_add: true },
            ],
            enemy_base_hp: 942,
            fire_rate: 10,
            last_fire: Instant::now(),
            last_tick: Instant::now(),
        }
    }

    fn reset_gameplay(&mut self) {
        self.mobs.clear();
        self.enemy_base_hp = 1000 + (self.level as i32 * 200);
        self.screen = Screen::Gameplay;
    }

    fn on_tick(&mut self) {
        if self.screen != Screen::Gameplay { return; }

        let now = Instant::now();
        self.last_tick = now;

        // Fire cannon
        if now.duration_since(self.last_fire).as_millis() > (1000 / self.fire_rate) as u128 {
            self.mobs.push(Mob { x: self.cannon_x as f64, y: 35.0, is_enemy: false });
            self.last_fire = now;
        }

        // Move mobs
        let mut new_mobs = Vec::new();
        let mut rng = rand::rng();
        
        for mob in &mut self.mobs {
            if mob.is_enemy {
                mob.y += 0.5;
            } else {
                mob.y -= 0.5;
            }

            if !mob.is_enemy {
                for gate in &self.gates {
                    if (mob.y as u16) == gate.y && (mob.x as u16) >= gate.x && (mob.x as u16) < gate.x + gate.width {
                        let count = if gate.is_add { gate.multiplier } else { gate.multiplier - 1 };
                        for _ in 0..count {
                            new_mobs.push(Mob { 
                                x: mob.x + rng.random_range(-1.0..1.0), 
                                y: mob.y, 
                                is_enemy: false 
                            });
                        }
                    }
                }
            }

            if !mob.is_enemy && mob.y <= 2.0 {
                self.enemy_base_hp -= 1;
                self.bits += 1;
                continue;
            }

            if mob.y > 0.0 && mob.y < 40.0 {
                new_mobs.push(Mob { x: mob.x, y: mob.y, is_enemy: mob.is_enemy });
            }
        }
        
        if rng.random_bool(0.05) {
             self.mobs.push(Mob { 
                x: rng.random_range(5.0..35.0), 
                y: 5.0, 
                is_enemy: true 
            });
        }

        let mut final_mobs = Vec::new();
        let (mut players, mut enemies): (Vec<_>, Vec<_>) = new_mobs.into_iter().partition(|m| !m.is_enemy);
        
        players.retain(|p| {
            let mut survived = true;
            enemies.retain(|e| {
                if survived && (p.x - e.x).abs() < 1.0 && (p.y - e.y).abs() < 1.0 {
                    survived = false;
                    false
                } else {
                    true
                }
            });
            survived
        });

        final_mobs.extend(players);
        final_mobs.extend(enemies);
        self.mobs = final_mobs;

        if self.enemy_base_hp <= 0 {
            self.screen = Screen::GameOver;
        }
    }
}

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
        terminal.draw(|f| ui(f, app)).map_err(|e| e.to_string())?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                match app.screen {
                    Screen::Menu => {
                        match key.code {
                            KeyCode::Enter => app.reset_gameplay(),
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
                                if app.bits >= 250 {
                                    app.bits -= 250;
                                    app.fire_rate += 2;
                                }
                            }
                            _ => {}
                        }
                    }
                    Screen::GameOver => {
                        if key.code == KeyCode::Enter {
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

fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    match app.screen {
        Screen::Menu => draw_menu(f, area, app),
        Screen::Gameplay => draw_gameplay(f, area, app),
        Screen::GameOver => draw_gameover(f, area, app),
        Screen::Upgrades => draw_upgrades(f, area, app),
    }
}

fn draw_upgrades(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(area);

    let header = Paragraph::new(format!("BITS_AVAIL: {}", app.bits))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(57, 255, 20)).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL).title(" [ BLACK_MARKET ] "));
    f.render_widget(header, chunks[0]);

    let upgrades = vec![
        Line::from("[UPG_01] EMISSION_RATE -> Increase fire rate (Cost: 250)").fg(Color::Rgb(0, 255, 204)),
        Line::from(format!("        Current: {}/s", app.fire_rate)),
        Line::from(""),
        Line::from("[UPG_02] PACKET_SIZE -> (Coming Soon)").fg(Color::Rgb(74, 74, 90)),
    ];
    let list = Paragraph::new(upgrades)
        .block(Block::default().borders(Borders::ALL).padding(ratatui::widgets::Padding::horizontal(2)));
    f.render_widget(list, chunks[1]);

    let footer = Paragraph::new("> PRESS 'B' TO RETURN TO TERMINAL <")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(176, 38, 255)));
    f.render_widget(footer, chunks[2]);
}

fn draw_menu(f: &mut Frame, area: Rect, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(5)
        .constraints([
            Constraint::Length(10),
            Constraint::Min(0),
            Constraint::Length(4),
        ])
        .split(area);

    let logo = vec![
        Line::from(" ███╗   ███╗ ██████╗ ██████╗     ██████╗████████╗██████╗ ██╗     "),
        Line::from(" ████╗ ████║██╔═══██╗██╔══██╗   ██╔════╝╚══██╔══╝██╔══██╗██║     "),
        Line::from(" ██╔████╔██║██║   ██║██████╔╝   ██║        ██║   ██████╔╝██║     "),
        Line::from(" ██║╚██╔╝██║██║   ██║██╔══██╗   ██║        ██║   ██╔══██╗██║     "),
        Line::from(" ██║ ╚═╝ ██║╚██████╔╝██████╔╝   ╚██████╗   ██║   ██║  ██║███████╗"),
        Line::from(" ╚═╝     ╚═╝ ╚═════╝ ╚═════╝     ╚═════╝   ╚═╝   ╚═╝  ╚═╝╚══════╝"),
    ];
    let logo_para = Paragraph::new(logo)
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(57, 255, 20)).add_modifier(Modifier::BOLD));
    f.render_widget(logo_para, chunks[0]);

    let prompt = Paragraph::new("[ENTER] TO EXECUTE_")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(0, 255, 204)));
    f.render_widget(prompt, chunks[1]);

    let footer = Paragraph::new(vec![
        Line::from(format!("BITS_MINED: {} | MAX_LEVEL: {}", app.bits, app.level)),
        Line::from("[T] BLACK_MARKET  |  [ENTER] EXEC_MODE"),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(Color::Rgb(176, 38, 255))));
    f.render_widget(footer, chunks[2]);
}

fn draw_gameplay(f: &mut Frame, area: Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);
    
    let header = Line::from(vec![
        Span::raw(" LVL ").fg(Color::Rgb(0, 255, 204)),
        Span::raw(format!("{:02}", app.level)).fg(Color::Rgb(57, 255, 20)),
        Span::raw(" ".repeat((area.width as usize).saturating_sub(25))),
        Span::raw("BITS: ").fg(Color::Rgb(0, 255, 204)),
        Span::raw(format!("{:04}", app.bits)).fg(Color::Rgb(176, 38, 255)),
    ]);
    f.render_widget(Paragraph::new(header), header_chunks[0]);

    let arena_rect = Rect::new(area.width / 2 - 20, 1, 40, area.height - 2);
    let arena_block = Block::default()
        .borders(Borders::LEFT | Borders::RIGHT)
        .border_set(symbols::border::DOUBLE)
        .border_style(Style::default().fg(Color::Rgb(176, 38, 255)));
    f.render_widget(arena_block, arena_rect);

    let base_y = 3;
    let base_width = 30;
    let base_rect = Rect::new(arena_rect.x + (arena_rect.width - base_width)/2, base_y, base_width, 2);
    let base_fill = Paragraph::new("█".repeat(base_width as usize))
        .style(Style::default().fg(Color::Rgb(255, 0, 60)));
    f.render_widget(base_fill, base_rect);
    
    let hp_text = Paragraph::new(format!("[ {} ]", app.enemy_base_hp))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(255, 0, 60)));
    f.render_widget(hp_text, Rect::new(base_rect.x, base_rect.y + 2, base_rect.width, 1));

    for gate in &app.gates {
        let gate_rect = Rect::new(arena_rect.x + gate.x, gate.y, gate.width, 3);
        let gate_text = if gate.is_add { format!("+{}", gate.multiplier) } else { format!("x{}", gate.multiplier) };
        let gate_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("[ {} ]", gate_text))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Rgb(0, 255, 204)));
        f.render_widget(gate_block, gate_rect);
    }

    for mob in &app.mobs {
        let x = (arena_rect.x as f64 + mob.x).min(arena_rect.x as f64 + arena_rect.width as f64 - 2.0) as u16;
        let y = mob.y as u16;
        if y >= arena_rect.y && y < arena_rect.y + arena_rect.height {
            let symbol = if mob.is_enemy { "x" } else { "*" };
            let color = if mob.is_enemy { Color::Rgb(255, 0, 60) } else { Color::Rgb(57, 255, 20) };
            f.render_widget(Paragraph::new(symbol).style(Style::default().fg(color)), Rect::new(x, y, 1, 1));
        }
    }

    let cannon_rect = Rect::new(arena_rect.x + app.cannon_x, arena_rect.y + arena_rect.height - 2, 3, 1);
    f.render_widget(Paragraph::new("▄█▄").style(Style::default().fg(Color::Rgb(57, 255, 20))), cannon_rect);
}

fn draw_gameover(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 0, 60)))
        .title(" [ SYS_HALTED ] ");
    
    let area = Rect::new(area.width / 2 - 20, area.height / 2 - 5, 40, 10);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let text = vec![
        Line::from("TARGET_DESTROYED").alignment(Alignment::Center).fg(Color::Rgb(57, 255, 20)),
        Line::from("").alignment(Alignment::Center),
        Line::from(format!("BITS_MINED: {}", app.bits)).alignment(Alignment::Center),
        Line::from("").alignment(Alignment::Center),
        Line::from("> PRESS ENTER TO REBOOT <").alignment(Alignment::Center).fg(Color::Rgb(57, 255, 20)),
    ];
    let para = Paragraph::new(text).block(Block::default().padding(ratatui::widgets::Padding::vertical(2)));
    f.render_widget(para, area);
}
