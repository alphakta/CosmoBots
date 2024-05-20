use ggez::graphics;
use ggez::{event, timer, Context, GameResult};
use noise::{Fbm, NoiseFn, Perlin};
use rand::Rng;
use std::time::Duration;

use crate::{RobotExplorer, RobotExtractor};
use crate::{CELL_SIZE, ENERGY_THRESHOLD, MAP_SIZE, MINERALS_THRESHOLD, OBSTACLE_THRESHOLD};
use crate::{
    DEFAULT_COLOR, ENERGY_COLOR, FOG_COLOR, MINERALS_COLOR, OBSTACLE_COLOR, ROBOT_EXPLORER_COLOR,
    ROBOT_EXTRACTOR_COLOR, SCIENCE_INTERESTS_COLOR, STATION_COLOR,
};

pub struct Map {
    pub energy: [[bool; MAP_SIZE]; MAP_SIZE],
    pub minerals: [[bool; MAP_SIZE]; MAP_SIZE],
    pub science_interests: [[bool; MAP_SIZE]; MAP_SIZE],
    pub obstacles: [[bool; MAP_SIZE]; MAP_SIZE],
    pub explored: [[bool; MAP_SIZE]; MAP_SIZE],
    pub fog_of_war: [[bool; MAP_SIZE]; MAP_SIZE],
    pub robot_explorer: Option<RobotExplorer>,
    pub robot_extractor: Option<RobotExtractor>,
    pub update_timer: Duration,
    pub game_over: bool,
    pub nb_consumables: usize,
    pub resources: Vec<(usize, usize)>,
}

impl Map {
    pub fn new() -> Self {
        let mut map = Map {
            energy: [[false; MAP_SIZE]; MAP_SIZE],
            minerals: [[false; MAP_SIZE]; MAP_SIZE],
            science_interests: [[false; MAP_SIZE]; MAP_SIZE],
            obstacles: [[false; MAP_SIZE]; MAP_SIZE],
            explored: [[false; MAP_SIZE]; MAP_SIZE],
            fog_of_war: [[true; MAP_SIZE]; MAP_SIZE],
            robot_explorer: None,
            robot_extractor: None,
            update_timer: Duration::from_secs(1),
            game_over: false,
            nb_consumables: 0,
            resources: vec![],
        };

        if let Some((x, y)) = map.init_robot_position() {
            map.robot_explorer = Some(RobotExplorer::new(x, y));
            map.robot_extractor = Some(RobotExtractor::new(x, y));
            map.fog_of_war[y][x] = false;
        }

        map
    }

    pub fn init_robot_position(&self) -> Option<(usize, usize)> {
        let mut rng = rand::thread_rng();
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 100;

        loop {
            let x = rng.gen_range(0..MAP_SIZE);
            let y = rng.gen_range(0..MAP_SIZE);

            if !self.obstacles[y][x] && !self.energy[y][x] && !self.minerals[y][x] {
                return Some((x, y));
            }

            attempts += 1;
            if attempts >= MAX_ATTEMPTS {
                return None;
            }
        }
    }

    pub fn place_obstacles(&mut self, _resources: &[(usize, usize)]) {
        let mut rng = rand::thread_rng();
        let seed = rng.gen();
        let fbm_obstacles = Fbm::<Perlin>::new(seed);

        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                if !self
                    .resources
                    .iter()
                    .any(|&(res_x, res_y)| res_x == x && res_y == y)
                {
                    let obstacles_noise = fbm_obstacles.get([x as f64, y as f64]);
                    self.obstacles[y][x] = obstacles_noise > OBSTACLE_THRESHOLD;
                }
            }
        }
    }

    pub fn place_resources(&mut self) {
        let mut rng = rand::thread_rng();
        let energy_seed = rng.gen();
        let minerals_seed = rng.gen();
        let fbm_energy = Fbm::<Perlin>::new(energy_seed);
        let fbm_minerals = Fbm::<Perlin>::new(minerals_seed);

        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let energy_noise = fbm_energy.get([x as f64, y as f64]);
                let minerals_noise = fbm_minerals.get([x as f64, y as f64]);
                self.energy[y][x] = energy_noise > ENERGY_THRESHOLD;
                self.minerals[y][x] = minerals_noise > MINERALS_THRESHOLD;

                if self.energy[y][x] || self.minerals[y][x] {
                    self.resources.push((x, y));
                }

                println!("Number of consumables: {}", self.resources.len());
            }
        }
    }

    pub fn update_robot(&mut self) {
        if self.game_over {
            return;
        }

        if let Some(mut extractor) = self.robot_extractor.take() {
            if !extractor.waiting {
                if extractor.carrying_resource {
                    if let Some(path) = extractor.move_towards(
                        extractor.station_x,
                        extractor.station_y,
                        &self.obstacles,
                    ) {
                        if path.len() > 1 {
                            let (next_x, next_y) = path[1];
                            extractor.x = next_x;
                            extractor.y = next_y;

                            if extractor.x == extractor.station_x
                                && extractor.y == extractor.station_y
                            {
                                extractor.carrying_resource = false;
                                extractor.waiting = true;
                                if let Some(mut explorer) = self.robot_explorer.as_mut() {
                                    explorer.waiting = false;
                                }
                                println!("Extractor returned to the station with resource. Remaining resources: {}", self.count_consumables());
                            }
                        }
                    }
                } else if let Some((target_x, target_y)) = extractor.target_position {
                    if let Some(path) = extractor.move_towards(target_x, target_y, &self.obstacles)
                    {
                        if path.len() > 1 {
                            let (next_x, next_y) = path[1];
                            extractor.x = next_x;
                            extractor.y = next_y;

                            if extractor.x == target_x && extractor.y == target_y {
                                extractor.collect_resource();
                                self.energy[target_y][target_x] = false;
                                self.minerals[target_y][target_x] = false;
                            }
                        }
                    }
                }
            }
            self.robot_extractor = Some(extractor);
        }

        if let Some(mut explorer) = self.robot_explorer.take() {
            if (!explorer.waiting) {
                let mut rng = rand::thread_rng();

                let all_resources_collected = self.count_consumables() == 0 || self.is_map_empty();

                if all_resources_collected {
                    if let Some(path) = explorer.return_to_station(&self.obstacles) {
                        if path.len() > 1 {
                            let (next_x, next_y) = path[1];
                            explorer.x = next_x;
                            explorer.y = next_y;

                            if explorer.x == explorer.station_x && explorer.y == explorer.station_y
                            {
                                self.game_over = true;
                                println!("Game Over: Robot returned to the station. All resources have been collected.");
                            }
                        }
                    }
                } else if explorer.founded_resource {
                    if let Some((resource_x, resource_y)) = explorer.resource_position {
                        if explorer.x == explorer.station_x && explorer.y == explorer.station_y {
                            self.robot_extractor.as_mut().unwrap().target_position =
                                Some((resource_x, resource_y));
                            explorer.founded_resource = false;
                            explorer.resource_position = None;
                            explorer.waiting = true;
                            if let Some(mut extractor) = self.robot_extractor.as_mut() {
                                extractor.waiting = false;
                            }
                            println!("Explorer returned to the station and provided resource position to Extractor.");
                        } else if let Some(path) = explorer.return_to_station(&self.obstacles) {
                            if path.len() > 1 {
                                let (next_x, next_y) = path[1];
                                explorer.x = next_x;
                                explorer.y = next_y;
                            }
                        }
                    }
                } else {
                    explorer.move_random(&mut rng, self);
                }
            }
            self.robot_explorer = Some(explorer);
        }
    }

    pub fn is_map_empty(&self) -> bool {
        !self.energy.iter().any(|row| row.iter().any(|&val| val))
            && !self.minerals.iter().any(|row| row.iter().any(|&val| val))
    }

    pub fn count_consumables(&self) -> usize {
        self.energy
            .iter()
            .flatten()
            .chain(self.minerals.iter().flatten())
            .filter(|&&val| val)
            .count()
    }
}

impl event::EventHandler for Map {
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::WHITE);

        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let rect = graphics::Rect::new(
                    x as f32 * CELL_SIZE as f32,
                    y as f32 * CELL_SIZE as f32,
                    CELL_SIZE as f32,
                    CELL_SIZE as f32,
                );
                let cell_color = if self.robot_explorer.as_ref().unwrap().station_x == x
                    && self.robot_explorer.as_ref().unwrap().station_y == y
                {
                    STATION_COLOR
                } else if self.obstacles[y][x] {
                    OBSTACLE_COLOR
                } else if self.energy[y][x] {
                    ENERGY_COLOR
                } else if self.minerals[y][x] {
                    MINERALS_COLOR
                } else if self.science_interests[y][x] {
                    SCIENCE_INTERESTS_COLOR
                } else if self.robot_explorer.as_ref().unwrap().x == x
                    && self.robot_explorer.as_ref().unwrap().y == y
                {
                    ROBOT_EXPLORER_COLOR
                } else if self.robot_extractor.as_ref().unwrap().x == x
                    && self.robot_extractor.as_ref().unwrap().y == y
                {
                    ROBOT_EXTRACTOR_COLOR
                } else {
                    DEFAULT_COLOR
                };

                let fog_color = if self.fog_of_war[y][x] {
                    FOG_COLOR
                } else {
                    cell_color
                };

                let cell = graphics::Mesh::new_rectangle(
                    ctx,
                    graphics::DrawMode::fill(),
                    rect,
                    fog_color,
                )?;
                graphics::draw(ctx, &cell, graphics::DrawParam::default())?;
            }
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        while timer::check_update_time(ctx, 1) {
            if self.update_timer.as_secs() == 0 {
                self.update_timer = Duration::from_secs(1);
            } else {
                self.update_timer = self
                    .update_timer
                    .checked_sub(Duration::from_secs(1))
                    .unwrap_or(Duration::from_secs(0));
            }

            self.update_robot();
        }
        Ok(())
    }
}
