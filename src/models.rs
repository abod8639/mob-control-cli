use std::time::{Instant, Duration};

#[derive(PartialEq, Clone, Copy)]
pub enum Screen {
    Menu,
    Gameplay,
    Upgrades,
    GameOver,
    LevelComplete,
}

pub struct Mob {
    pub x: f64,
    pub y: f64,
    pub is_enemy: bool,
    pub hp: i32,
    pub speed: f64,
}

pub struct Obstacle {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64,
    pub is_slanted: bool,
}

pub struct Gate {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub multiplier: u32,
    pub is_add: bool,
}

pub struct App {
    pub screen: Screen,
    pub bits: u32,
    pub level: u32,
    pub cannon_pos: f64, // 0.0 to 1.0 along the track
    pub cannon_x: u16,
    pub cannon_y: u16,
    pub mobs: Vec<Mob>,
    pub gates: Vec<Gate>,
    pub obstacles: Vec<Obstacle>,
    pub enemy_base_hp: i32,
    pub enemy_base_max_hp: i32,
    pub fire_rate: u32,
    pub mob_packet_size: u32,
    pub mob_speed: f64,
    pub mob_damage: i32,
    pub current_wave: u32,
    pub total_waves: u32,
    pub enemies_to_spawn: u32,
    pub wave_incoming: bool,
    pub spawn_cooldown: Duration,
    pub last_enemy_spawn: Instant,
    pub wave_timer: Instant,
    pub last_fire: Instant,
    pub last_tick: Instant,
}
