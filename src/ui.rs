use crate::models::{App, Screen};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    symbols,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
};

pub fn ui(f: &mut Frame, app: &App) {
    let area = f.area();

    match app.screen {
        Screen::Menu => draw_menu(f, area, app),
        Screen::Gameplay => draw_gameplay(f, area, app),
        Screen::GameOver => draw_gameover(f, area, app),
        Screen::Upgrades => draw_upgrades(f, area, app),
        Screen::LevelComplete => draw_level_complete(f, area, app),
    }
}

pub fn draw_upgrades(f: &mut Frame, area: Rect, app: &App) {
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
        .style(
            Style::default()
                .fg(Color::Rgb(57, 255, 20))
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" [ BLACK_MARKET ] "),
        );
    f.render_widget(header, chunks[0]);

    let upgrades = vec![
        Line::from(format!(
            "[1] EMISSION_RATE  -> Speed up cannon (Cost: {})",
            100 + (app.fire_rate as u32 * 10)
        ))
        .fg(Color::Rgb(0, 255, 204)),
        Line::from(format!("    Current: {:.1}/s", app.fire_rate as f64 / 10.0)),
        Line::from(""),
        Line::from(format!(
            "[2] PACKET_SIZE    -> Mobs per shot  (Cost: {})",
            app.mob_packet_size * 500
        ))
        .fg(Color::Rgb(0, 255, 204)),
        Line::from(format!("    Current: x{}", app.mob_packet_size)),
        Line::from(""),
        Line::from(format!(
            "[3] SIGNAL_STRENGTH-> Mob movement   (Cost: {})",
            300 + (app.mob_speed * 1000.0) as u32
        ))
        .fg(Color::Rgb(0, 255, 204)),
        Line::from(format!("    Current: {:.1} units", app.mob_speed)),
        Line::from(""),
        Line::from(format!(
            "[4] DATA_CORRUPTION-> Base Damage    (Cost: {})",
            app.mob_damage as u32 * 1000
        ))
        .fg(Color::Rgb(0, 255, 204)),
        Line::from(format!("    Current: {} DMG", app.mob_damage)),
    ];
    let list = Paragraph::new(upgrades).block(
        Block::default()
            .borders(Borders::ALL)
            .padding(ratatui::widgets::Padding::horizontal(2)),
    );
    f.render_widget(list, chunks[1]);

    let footer = Paragraph::new("> PRESS 'B' TO RETURN TO TERMINAL <")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(176, 38, 255)));
    f.render_widget(footer, chunks[2]);
}

pub fn draw_menu(f: &mut Frame, area: Rect, app: &App) {
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
    let logo_para = Paragraph::new(logo).alignment(Alignment::Center).style(
        Style::default()
            .fg(Color::Rgb(57, 255, 20))
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(logo_para, chunks[0]);

    let prompt = Paragraph::new(format!("[ENTER] TO START LEVEL {:02}_", app.level))
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Rgb(0, 255, 204)));
    f.render_widget(prompt, chunks[1]);

    let footer = Paragraph::new(vec![
        Line::from(format!(
            "BITS_MINED: {} | CURRENT_TARGET: LEVEL {:02}",
            app.bits, app.level
        )),
        Line::from("[T] BLACK_MARKET  |  [ENTER] EXEC_MODE"),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::TOP)
            .border_style(Style::default().fg(Color::Rgb(176, 38, 255))),
    );
    f.render_widget(footer, chunks[2]);
}

pub fn draw_gameplay(f: &mut Frame, area: Rect, app: &App) {
    let header_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Min(0)])
        .split(area);

    let wave_info = if app.wave_incoming {
        format!("PREPARING WAVE {}...", app.current_wave + 1)
    } else if app.enemies_to_spawn > 0 {
        format!("WAVE {}/{} (ENEMIES: {})", app.current_wave, app.total_waves, app.enemies_to_spawn)
    } else if app.current_wave >= app.total_waves {
        "FINAL_STRETCH".to_string()
    } else {
        format!("WAVE {}/{} CLEAR!", app.current_wave, app.total_waves)
    };

    let header = Line::from(vec![
        Span::raw(" LVL ").fg(Color::Rgb(0, 255, 204)),
        Span::raw(format!("{:02}", app.level)).fg(Color::Rgb(57, 255, 20)),
        Span::raw(" | ").fg(Color::Rgb(74, 74, 90)),
        Span::raw(wave_info).fg(if app.wave_incoming { Color::Rgb(255, 0, 60) } else { Color::Rgb(255, 255, 0) }),
        Span::raw(" ".repeat((area.width as usize).saturating_sub(60))),
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
    let base_rect = Rect::new(
        arena_rect.x + (arena_rect.width - base_width) / 2,
        base_y,
        base_width,
        2,
    );

    let hp_ratio = (app.enemy_base_hp as f32 / app.enemy_base_max_hp as f32).max(0.0);
    let filled_width = (base_width as f32 * hp_ratio) as usize;
    let hp_bar = format!(
        "{}{}",
        "█".repeat(filled_width),
        "░".repeat(base_width as usize - filled_width)
    );

    let base_fill = Paragraph::new(hp_bar).style(Style::default().fg(Color::Rgb(255, 0, 60)));
    f.render_widget(base_fill, base_rect);

    let hp_text = Paragraph::new(format!(
        "[ {} / {} ]",
        app.enemy_base_hp, app.enemy_base_max_hp
    ))
    .alignment(Alignment::Center)
    .style(Style::default().fg(Color::Rgb(255, 0, 60)));
    f.render_widget(
        hp_text,
        Rect::new(base_rect.x, base_rect.y + 2, base_rect.width, 1),
    );

    for gate in &app.gates {
        let gate_rect = Rect::new(arena_rect.x + gate.x, gate.y, gate.width, 3);
        let gate_text = if gate.is_add {
            format!("+{}", gate.multiplier)
        } else {
            format!("x{}", gate.multiplier)
        };
        let gate_block = Block::default()
            .borders(Borders::ALL)
            .title(format!("[ {} ]", gate_text))
            .title_alignment(Alignment::Center)
            .border_style(Style::default().fg(Color::Rgb(0, 255, 204)));
        f.render_widget(gate_block, gate_rect);
    }

    for mob in &app.mobs {
        let x = (arena_rect.x as f64 + mob.x).clamp(
            arena_rect.x as f64 + 1.0,
            arena_rect.x as f64 + arena_rect.width as f64 - 2.0,
        ) as u16;
        let y = mob.y as u16;
        if y >= arena_rect.y && y < arena_rect.y + arena_rect.height {
            let symbol = if mob.is_enemy { "▼" } else { "▲" };
            let color = if mob.is_enemy {
                Color::Rgb(255, 0, 60)
            } else {
                Color::Rgb(57, 255, 20)
            };
            f.render_widget(
                Paragraph::new(symbol).style(Style::default().fg(color)),
                Rect::new(x, y, 1, 1),
            );
        }
    }

    let cannon_rect = Rect::new(
        arena_rect.x + app.cannon_x,
        arena_rect.y + arena_rect.height - 2,
        3,
        1,
    );
    f.render_widget(
        Paragraph::new("︻╦╤─").style(Style::default().fg(Color::Rgb(57, 255, 20))),
        cannon_rect,
    );
}

pub fn draw_level_complete(f: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(0, 255, 204)))
        .title(" [ LEVEL_CLEAR ] ");

    let area = Rect::new(area.width / 2 - 20, area.height / 2 - 5, 40, 10);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let text = vec![
        Line::from(format!("LEVEL {:02} COMPLETED", app.level))
            .alignment(Alignment::Center)
            .fg(Color::Rgb(57, 255, 20)),
        Line::from("").alignment(Alignment::Center),
        Line::from("NODE_ACCESS_GRANTED").alignment(Alignment::Center),
        Line::from(format!("TOTAL_BITS: {}", app.bits)).alignment(Alignment::Center),
        Line::from("").alignment(Alignment::Center),
        Line::from("> PRESS ENTER TO ADVANCE <")
            .alignment(Alignment::Center)
            .fg(Color::Rgb(57, 255, 20)),
    ];
    let para = Paragraph::new(text)
        .block(Block::default().padding(ratatui::widgets::Padding::vertical(2)));
    f.render_widget(para, area);
}

pub fn draw_gameover(f: &mut Frame, area: Rect, _app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Rgb(255, 0, 60)))
        .title(" [ SYS_HALTED ] ");

    let area = Rect::new(area.width / 2 - 20, area.height / 2 - 5, 40, 10);
    f.render_widget(Clear, area);
    f.render_widget(block, area);

    let text = vec![
        Line::from("CONNECTION_LOST")
            .alignment(Alignment::Center)
            .fg(Color::Rgb(255, 0, 60)),
        Line::from("").alignment(Alignment::Center),
        Line::from("HARD_REBOOT_REQUIRED").alignment(Alignment::Center),
        Line::from("").alignment(Alignment::Center),
        Line::from("> PRESS ENTER TO RESTART <")
            .alignment(Alignment::Center)
            .fg(Color::Rgb(255, 0, 60)),
    ];
    let para = Paragraph::new(text)
        .block(Block::default().padding(ratatui::widgets::Padding::vertical(2)));
    f.render_widget(para, area);
}
