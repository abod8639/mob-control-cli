use crate::models::{App, Mob, Gate, Obstacle, Screen};
use rand::RngExt;
use std::time::{Instant, Duration};

impl App {
    pub fn new() -> App {
        App {
            screen: Screen::Menu,
            bits: 0,
            level: 1,
            cannon_pos: 0.5,
            cannon_x: 20,
            cannon_y: 36,
            mobs: Vec::new(),
            gates: Vec::new(),
            obstacles: Vec::new(),
            enemy_base_hp: 1000,
            enemy_base_max_hp: 1000,
            fire_rate: 20,
            mob_packet_size: 1,
            mob_speed: 0.5,
            mob_damage: 1,
            current_wave: 1,
            total_waves: 3,
            enemies_to_spawn: 0,
            wave_incoming: false,
            spawn_cooldown: Duration::from_millis(500),
            last_enemy_spawn: Instant::now(),
            wave_timer: Instant::now(),
            last_fire: Instant::now(),
            last_tick: Instant::now(),
        }
    }

    pub fn init_level(&mut self) {
        self.mobs.clear();
        self.current_wave = 0;
        self.total_waves = 3 + (self.level / 2);
        self.enemy_base_max_hp = 500 + (self.level as i32 * 500);
        self.enemy_base_hp = self.enemy_base_max_hp;
        self.enemies_to_spawn = 0;
        self.wave_incoming = true;

        let mut rng = rand::rng();
        
        // 1. Generate Obstacles - MOVED TO TOP (y: 6 to 18)
        // Enemies will have to navigate these first
        self.obstacles.clear();
        let pattern = (self.level + rng.random_range(0..10)) % 4;
        match pattern {
            0 => { // Lateral Slanted Funnel at Top
                self.obstacles.push(Obstacle { x1: 2.0, y1: 18.0, x2: 12.0, y2: 10.0, is_slanted: true });
                self.obstacles.push(Obstacle { x1: 38.0, y1: 18.0, x2: 28.0, y2: 10.0, is_slanted: true });
            }
            1 => { // Central Horizontal Blockers at Top
                self.obstacles.push(Obstacle { x1: 10.0, y1: 14.0, x2: 30.0, y2: 14.0, is_slanted: false });
            }
            2 => { // Diamond Shape at Top
                self.obstacles.push(Obstacle { x1: 20.0, y1: 18.0, x2: 10.0, y2: 12.0, is_slanted: true });
                self.obstacles.push(Obstacle { x1: 20.0, y1: 18.0, x2: 30.0, y2: 12.0, is_slanted: true });
            }
            _ => { // Top Walls
                self.obstacles.push(Obstacle { x1: 0.0, y1: 12.0, x2: 15.0, y2: 12.0, is_slanted: false });
                self.obstacles.push(Obstacle { x1: 25.0, y1: 12.0, x2: 40.0, y2: 12.0, is_slanted: false });
            }
        }

        // 2. Generate Gates - MOVED NEAR CANNON (y: 22 to 30)
        // Player's mobs will multiply early for a bigger army feel
        self.gates.clear();
        let num_gates = 1 + (self.level % 2);
        for i in 0..num_gates {
            let is_add = rng.random_bool(0.4);
            let mult = if is_add { 5 + self.level * 2 } else { 2 + (self.level / 5) };
            self.gates.push(Gate {
                x: rng.random_range(5..30),
                y: 22 + (i as u16 * 5),
                width: 10 + rng.random_range(0..5),
                multiplier: mult,
                is_add,
            });
        }

        self.wave_timer = Instant::now();
        self.update_cannon_pos();
        self.screen = Screen::Gameplay;
    }

    fn update_cannon_pos(&mut self) {
        let x = 5.0 + (self.cannon_pos * 30.0);
        self.cannon_x = x as u16;
        self.cannon_y = 36;
    }

    pub fn move_cannon(&mut self, delta: f64) {
        self.cannon_pos = (self.cannon_pos + delta).clamp(0.0, 1.0);
        self.update_cannon_pos();
    }

    pub fn on_tick(&mut self) {
        if self.screen != Screen::Gameplay { return; }

        let now = Instant::now();
        self.last_tick = now;

        // Wave management
        if self.enemies_to_spawn == 0 {
            let time_since_last_wave = now.duration_since(self.wave_timer).as_secs();
            if self.wave_incoming && time_since_last_wave > 5 {
                self.current_wave += 1;
                self.enemies_to_spawn = 10 + (self.level * 5) + (self.current_wave * 5);
                self.wave_incoming = false;
                let ms = (800 / (1 + self.level)).max(150) as u64;
                self.spawn_cooldown = Duration::from_millis(ms);
                self.wave_timer = now;
            } else if !self.wave_incoming && self.current_wave < self.total_waves {
                let enemies_alive = self.mobs.iter().filter(|m| m.is_enemy).count();
                if enemies_alive == 0 {
                    self.wave_incoming = true;
                    self.wave_timer = now;
                }
            }
        }

        // Enemy spawning
        if self.enemies_to_spawn > 0 && now.duration_since(self.last_enemy_spawn) > self.spawn_cooldown {
            let mut rng = rand::rng();
            self.mobs.push(Mob {
                x: rng.random_range(15.0..25.0),
                y: 2.0,
                is_enemy: true,
                hp: 1 + (self.level / 3) as i32,
                speed: 0.2 + (self.level as f64 * 0.02),
            });
            self.enemies_to_spawn -= 1;
            self.last_enemy_spawn = now;
        }

        // Fire cannon
        if now.duration_since(self.last_fire).as_millis() > (5000 / self.fire_rate) as u128 {
            for i in 0..self.mob_packet_size {
                self.mobs.push(Mob { 
                    x: self.cannon_x as f64 + (i as f64 * 0.5) - (self.mob_packet_size as f64 * 0.25), 
                    y: self.cannon_y as f64 - 1.0, 
                    is_enemy: false,
                    hp: 1,
                    speed: self.mob_speed,
                });
            }
            self.last_fire = now;
        }

        let mut new_mobs = Vec::new();
        let mut rng = rand::rng();
        let mut processed_mobs = Vec::new();

        for mut mob in std::mem::take(&mut self.mobs) {
            let mut x_push = 0.0;
            let mut y_blocked = false;

            // --- Obstacle Collision Logic ---
            for obs in &self.obstacles {
                let y_min = obs.y1.min(obs.y2);
                let y_max = obs.y1.max(obs.y2);
                
                if mob.y >= y_min - 0.8 && mob.y <= y_max + 0.8 {
                    let target_x = if obs.is_slanted {
                         let ratio = ((mob.y - y_min) / (y_max - y_min).max(0.1)).clamp(0.0, 1.0);
                         if obs.y1 > obs.y2 {
                             obs.x2 + (obs.x1 - obs.x2) * ratio
                         } else {
                             obs.x1 + (obs.x2 - obs.x1) * ratio
                         }
                    } else {
                        if mob.x >= obs.x1.min(obs.x2) - 1.0 && mob.x <= obs.x1.max(obs.x2) + 1.0 {
                            mob.x
                        } else {
                            -100.0
                        }
                    };
                    
                    if (mob.x - target_x).abs() < 2.2 {
                        if obs.is_slanted {
                            if obs.x1 > obs.x2 { x_push += 0.45; } else { x_push -= 0.45; }
                        } else {
                            y_blocked = true;
                            if mob.x < (obs.x1 + obs.x2) / 2.0 { x_push -= 0.25; } else { x_push += 0.25; }
                        }
                    }
                }
            }

            if mob.is_enemy {
                if !y_blocked { mob.y += mob.speed; }
                mob.x = (mob.x + x_push).clamp(1.0, 39.0);
            } else {
                if !y_blocked { mob.y -= mob.speed; }
                mob.x = (mob.x + x_push).clamp(1.0, 39.0);

                // Gate check
                for gate in &self.gates {
                    if (mob.y as u16) == gate.y && (mob.x as u16) >= gate.x && (mob.x as u16) < gate.x + gate.width {
                        let count = if gate.is_add { gate.multiplier } else { mob.hp as u32 * (gate.multiplier - 1) };
                        for _ in 0..count {
                            new_mobs.push(Mob { 
                                x: (mob.x + rng.random_range(-1.5..1.5)).clamp(1.0, 39.0), 
                                y: mob.y, 
                                is_enemy: false,
                                hp: 1,
                                speed: mob.speed,
                            });
                        }
                    }
                }
            }

            if !mob.is_enemy && mob.y <= 3.5 {
                self.enemy_base_hp -= self.mob_damage;
                self.bits += self.level * 5;
                continue;
            }
            
            if mob.is_enemy && mob.y >= 38.0 {
                self.screen = Screen::GameOver;
                return;
            }

            if mob.y > 0.0 && mob.y < 40.0 {
                processed_mobs.push(mob);
            }
        }
        processed_mobs.extend(new_mobs);

        // Collisions between mobs
        let mut final_mobs = Vec::new();
        let (mut players, mut enemies): (Vec<_>, Vec<_>) = processed_mobs.into_iter().partition(|m| !m.is_enemy);
        
        players.retain(|p| {
            let mut survived = true;
            enemies.retain(|e| {
                if survived && (p.x - e.x).abs() < 1.2 && (p.y - e.y).abs() < 1.2 {
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
            self.bits += self.level * 1000;
            self.screen = Screen::LevelComplete;
        }
    }
}
