use crate::models::{App, Mob, Gate, Screen};
use rand::RngExt;
use std::time::{Instant, Duration};

impl App {
    pub fn new() -> App {
        App {
            screen: Screen::Menu,
            bits: 0,
            level: 1,
            cannon_x: 20,
            mobs: Vec::new(),
            gates: Vec::new(),
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
        self.gates.clear();
        let num_gates = 1 + (self.level % 3);
        for i in 0..num_gates {
            let is_add = rng.random_bool(0.4);
            let mult = if is_add {
                5 + self.level * 2
            } else {
                2 + (self.level / 5)
            };
            self.gates.push(Gate {
                x: rng.random_range(5..30),
                y: 10 + (i as u16 * 8),
                width: 8 + rng.random_range(0..5),
                multiplier: mult,
                is_add,
            });
        }
        self.wave_timer = Instant::now();
        self.screen = Screen::Gameplay;
    }

    pub fn on_tick(&mut self) {
        if self.screen != Screen::Gameplay {
            return;
        }

        let now = Instant::now();
        self.last_tick = now;

        // Wave management - Logical Progression
        if self.enemies_to_spawn == 0 {
            let time_since_last_wave = now.duration_since(self.wave_timer).as_secs();
            if self.wave_incoming && time_since_last_wave > 5 {
                // Start a new wave
                self.current_wave += 1;
                self.enemies_to_spawn = 10 + (self.level * 5) + (self.current_wave * 5);
                self.wave_incoming = false;
                // Spawning gets faster in later waves and levels
                let ms = (800 / (1 + self.level)).max(150) as u64;
                self.spawn_cooldown = Duration::from_millis(ms);
                self.wave_timer = now;
            } else if !self.wave_incoming && self.current_wave < self.total_waves {
                // Check if all enemies are cleared before starting next wave timer
                let enemies_alive = self.mobs.iter().filter(|m| m.is_enemy).count();
                if enemies_alive == 0 {
                    self.wave_incoming = true;
                    self.wave_timer = now;
                }
            }
        }

        // Gradual enemy spawning
        if self.enemies_to_spawn > 0 && now.duration_since(self.last_enemy_spawn) > self.spawn_cooldown {
            let mut rng = rand::rng();
            // Cluster spawning - sometimes spawn 2 at once for intensity
            let spawn_count = if rng.random_bool(0.15) { 2 } else { 1 };
            for _ in 0..spawn_count {
                if self.enemies_to_spawn > 0 {
                    self.mobs.push(Mob {
                        x: rng.random_range(5.0..35.0),
                        y: 3.0,
                        is_enemy: true,
                        hp: 1 + (self.level / 3) as i32,
                        speed: 0.2 + (self.level as f64 * 0.02) + (rng.random_range(0.0..0.1)),
                    });
                    self.enemies_to_spawn -= 1;
                }
            }
            self.last_enemy_spawn = now;
        }

        // Fire cannon
        if now.duration_since(self.last_fire).as_millis() > (3000 / self.fire_rate) as u128 {
            for i in 0..self.mob_packet_size {
                self.mobs.push(Mob {
                    x: self.cannon_x as f64 + (i as f64 * 0.5)
                        - (self.mob_packet_size as f64 * 0.25),
                    y: 35.0,
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
            if mob.is_enemy {
                mob.y += mob.speed;
            } else {
                mob.y -= mob.speed;

                for gate in &self.gates {
                    if (mob.y as u16) == gate.y
                        && (mob.x as u16) >= gate.x
                        && (mob.x as u16) < gate.x + gate.width
                    {
                        let count = if gate.is_add {
                            gate.multiplier
                        } else {
                            mob.hp as u32 * (gate.multiplier - 1)
                        };
                        for _ in 0..count {
                            new_mobs.push(Mob {
                                x: mob.x + rng.random_range(-1.5..1.5),
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
                self.bits += self.level * 5; // Fixed reward per hit
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

        let mut final_mobs = Vec::new();
        let (mut players, mut enemies): (Vec<_>, Vec<_>) =
            processed_mobs.into_iter().partition(|m| !m.is_enemy);

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
