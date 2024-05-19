use ggez::ContextBuilder;
use ggez::{conf, event, graphics, timer, Context, GameResult};
use lazy_static::lazy_static;
use noise::{Fbm, NoiseFn, Perlin};
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;

const MAP_SIZE: usize = 10;
const CELL_SIZE: f32 = 30.0;

const OBSTACLE_THRESHOLD: f64 = 0.2;
const ENERGY_THRESHOLD: f64 = 0.5;
const MINERALS_THRESHOLD: f64 = 0.5;
// const SCIENCE_INTERESTS_THRESHOLD: f64 = 0.5;

const DIRECTIONS: [(isize, isize); 8] = [
    (0, -1),
    (0, 1),
    (-1, 0),
    (1, 0),
    (-1, -1),
    (1, -1),
    (-1, 1),
    (1, 1),
];

lazy_static! {
    static ref DEFAULT_COLOR: graphics::Color = graphics::Color::from_rgb(255, 255, 255);
    static ref OBSTACLE_COLOR: graphics::Color = graphics::Color::from_rgb(127, 127, 127);
    static ref ENERGY_COLOR: graphics::Color = graphics::Color::from_rgb(255, 255, 0);
    static ref MINERALS_COLOR: graphics::Color = graphics::Color::from_rgb(0, 0, 255);
    static ref SCIENCE_INTERESTS_COLOR: graphics::Color = graphics::Color::from_rgb(0, 255, 0);
    static ref ROBOT_COLOR: graphics::Color = graphics::Color::from_rgb(255, 0, 0);
    static ref STATION_COLOR: graphics::Color = graphics::Color::from_rgb(0, 255, 255);
    static ref FOG_COLOR: graphics::Color = graphics::Color::from_rgb(0, 0, 0);
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct Node {
    x: usize,
    y: usize,
    cost: usize,
    priority: usize,
}

impl Node {
    fn new(x: usize, y: usize, cost: usize, priority: usize) -> Self {
        Node {
            x,
            y,
            cost,
            priority,
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

struct Map {
    energy: [[bool; MAP_SIZE]; MAP_SIZE],
    minerals: [[bool; MAP_SIZE]; MAP_SIZE],
    science_interests: [[bool; MAP_SIZE]; MAP_SIZE],
    obstacles: [[bool; MAP_SIZE]; MAP_SIZE],
    explored: [[bool; MAP_SIZE]; MAP_SIZE],
    fog_of_war: [[bool; MAP_SIZE]; MAP_SIZE],
    robot_explorer: Option<RobotExplorer>,
    update_timer: Duration,
    game_over: bool,
    nb_consumables: usize,
    resources: Vec<(usize, usize)>,
}

struct RobotExplorer {
    x: usize,
    y: usize,
    station_x: usize,
    station_y: usize,
    carrying_resource: bool,
}

struct RobotExtractor {
    x: usize,
    y: usize,
    station_x: usize,
    station_y: usize,
    carrying_resource: bool,
}

impl RobotExplorer {
    fn new(station_x: usize, station_y: usize) -> Self {
        RobotExplorer {
            x: station_x,
            y: station_y,
            station_x,
            station_y,
            carrying_resource: false,
        }
    }

    fn move_random(&mut self, rng: &mut impl Rng, map: &mut Map) {
        let mut possible_moves = vec![];

        for &(dx, dy) in &DIRECTIONS {
            let new_x = (self.x as isize + dx).max(0).min(MAP_SIZE as isize - 1) as usize;
            let new_y = (self.y as isize + dy).max(0).min(MAP_SIZE as isize - 1) as usize;
            if !map.obstacles[new_y][new_x] && !map.explored[new_y][new_x] {
                possible_moves.push((new_x, new_y));
            }
        }

        if possible_moves.is_empty() {
            for &(dx, dy) in &DIRECTIONS {
                let new_x = (self.x as isize + dx).max(0).min(MAP_SIZE as isize - 1) as usize;
                let new_y = (self.y as isize + dy).max(0).min(MAP_SIZE as isize - 1) as usize;
                if !map.obstacles[new_y][new_x] {
                    possible_moves.push((new_x, new_y));
                }
            }
        }

        if let Some(&(new_x, new_y)) = possible_moves.choose(rng) {
            self.x = new_x;
            self.y = new_y;
            map.explored[new_y][new_x] = true;

            for dy in -1..=1 {
                for dx in -1..=1 {
                    let neighbor_x =
                        (new_x as isize + dx).max(0).min(MAP_SIZE as isize - 1) as usize;
                    let neighbor_y =
                        (new_y as isize + dy).max(0).min(MAP_SIZE as isize - 1) as usize;
                    map.fog_of_war[neighbor_y][neighbor_x] = false;
                }
            }

            if map.energy[new_y][new_x] {
                map.energy[new_y][new_x] = false;
                self.carrying_resource = true;
                println!(
                    "Collected energy at ({}, {}). Remaining: {}",
                    new_x,
                    new_y,
                    map.count_consumables()
                );
            } else if map.minerals[new_y][new_x] {
                map.minerals[new_y][new_x] = false;
                self.carrying_resource = true;
                println!(
                    "Collected minerals at ({}, {}). Remaining: {}",
                    new_x,
                    new_y,
                    map.count_consumables()
                );
            }
        }
    }

    fn return_to_station(
        &mut self,
        obstacles: &[[bool; MAP_SIZE]; MAP_SIZE],
    ) -> Option<Vec<(usize, usize)>> {
        let start = (self.x, self.y);
        let goal = (self.station_x, self.station_y);

        let mut open_list = BinaryHeap::new();
        let mut came_from = HashMap::new();
        let mut g_score = vec![vec![usize::MAX; MAP_SIZE]; MAP_SIZE];
        let mut f_score = vec![vec![usize::MAX; MAP_SIZE]; MAP_SIZE];

        g_score[start.1][start.0] = 0;
        f_score[start.1][start.0] = heuristic(start.0, start.1, goal.0, goal.1);

        open_list.push(Node::new(start.0, start.1, 0, f_score[start.1][start.0]));

        while let Some(current) = open_list.pop() {
            if current.x == goal.0 && current.y == goal.1 {
                let mut path = vec![];
                let mut current_pos = (current.x, current.y);
                while let Some(&prev) = came_from.get(&current_pos) {
                    path.push(current_pos);
                    current_pos = prev;
                }
                path.push(start);
                path.reverse();
                return Some(path);
            }

            for &(dx, dy) in &DIRECTIONS {
                let neighbor_x =
                    (current.x as isize + dx).max(0).min(MAP_SIZE as isize - 1) as usize;
                let neighbor_y =
                    (current.y as isize + dy).max(0).min(MAP_SIZE as isize - 1) as usize;
                if obstacles[neighbor_y][neighbor_x] {
                    continue;
                }
                let tentative_g_score = g_score[current.y][current.x] + 1;
                if tentative_g_score < g_score[neighbor_y][neighbor_x] {
                    came_from.insert((neighbor_x, neighbor_y), (current.x, current.y));
                    g_score[neighbor_y][neighbor_x] = tentative_g_score;
                    f_score[neighbor_y][neighbor_x] =
                        tentative_g_score + heuristic(neighbor_x, neighbor_y, goal.0, goal.1);
                    open_list.push(Node::new(
                        neighbor_x,
                        neighbor_y,
                        tentative_g_score,
                        f_score[neighbor_y][neighbor_x],
                    ));
                }
            }
        }

        None
    }
}

impl Map {
    fn new() -> Self {
        let mut map = Map {
            energy: [[false; MAP_SIZE]; MAP_SIZE],
            minerals: [[false; MAP_SIZE]; MAP_SIZE],
            science_interests: [[false; MAP_SIZE]; MAP_SIZE],
            obstacles: [[false; MAP_SIZE]; MAP_SIZE],
            explored: [[false; MAP_SIZE]; MAP_SIZE],
            fog_of_war: [[true; MAP_SIZE]; MAP_SIZE],
            robot_explorer: None, // Initialisez le robot à None pour l'instant
            update_timer: Duration::from_secs(1),
            game_over: false,
            nb_consumables: 0,
            resources: vec![], // Initialisez le champ des ressources
        };

        // Initialisez la position du robot en appelant la fonction init_robot_position
        if let Some((x, y)) = map.init_robot_position() {
            map.robot_explorer = Some(RobotExplorer::new(x, y));
            map.fog_of_war[y][x] = false;
        }

        map
    }

    fn init_robot_position(&self) -> Option<(usize, usize)> {
        let mut rng = rand::thread_rng();
        let mut attempts = 0;
        const MAX_ATTEMPTS: usize = 100; // Nombre maximal de tentatives

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

    fn place_obstacles(&mut self, resources: &[(usize, usize)]) {
        let mut rng = rand::thread_rng();
        let seed = rng.gen();
        let fbm_obstacles = Fbm::<Perlin>::new(seed);

        // Utilisez les emplacements des ressources pour éviter de placer des obstacles à des emplacements similaires
        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                // Vérifiez si l'emplacement actuel est déjà occupé par une ressource
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

    fn place_resources(&mut self) {
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

    fn update_robot(&mut self) {
        if self.game_over {
            return;
        }

        if let Some(mut robot) = self.robot_explorer.take() {
            let mut rng = rand::thread_rng();

            // Si toutes les ressources sont collectées, mais avant de finir le jeu, le robot retourne à la station
            let all_resources_collected = self.count_consumables() == 0 || self.is_map_empty();

            if all_resources_collected && !robot.carrying_resource {
                if let Some(path) = robot.return_to_station(&self.obstacles) {
                    if path.len() > 1 {
                        // Obtenez la prochaine cellule dans le chemin
                        let (next_x, next_y) = path[1]; // Utilisez path[1] pour la prochaine cellule
                        robot.x = next_x;
                        robot.y = next_y;

                        if robot.x == robot.station_x && robot.y == robot.station_y {
                            self.game_over = true;
                            println!("Game Over: Robot returned to the station. All resources have been collected.");
                        }
                    }
                }
            } else {
                if robot.carrying_resource {
                    if let Some(path) = robot.return_to_station(&self.obstacles) {
                        if path.len() > 1 {
                            // Obtenez la prochaine cellule dans le chemin
                            let (next_x, next_y) = path[1]; // Utilisez path[1] pour la prochaine cellule
                            robot.x = next_x;
                            robot.y = next_y;

                            if robot.x == robot.station_x && robot.y == robot.station_y {
                                robot.carrying_resource = false; // Déposer la ressource à la station
                                println!("Resource delivered to the station.");
                            }
                        }
                    } else {
                        robot.move_random(&mut rng, self); // Aucun chemin trouvé, se déplacer aléatoirement
                    }
                } else {
                    robot.move_random(&mut rng, self); // Le robot ne porte pas de ressource, se déplacer aléatoirement
                }
            }

            // Mettre à jour le statut du robot
            self.robot_explorer = if self.game_over { None } else { Some(robot) };
        }
    }

    fn is_map_empty(&self) -> bool {
        !self.energy.iter().any(|row| row.iter().any(|&val| val))
            && !self.minerals.iter().any(|row| row.iter().any(|&val| val))
    }

    fn count_consumables(&self) -> usize {
        self.energy
            .iter()
            .flatten()
            .chain(self.minerals.iter().flatten())
            .filter(|&&val| val)
            .count()
    }
}

fn heuristic(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
    ((x2 as isize - x1 as isize).abs() + (y1 as isize - y2 as isize).abs()) as usize
}

impl event::EventHandler for Map {
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, *DEFAULT_COLOR);

        for y in 0..MAP_SIZE {
            for x in 0..MAP_SIZE {
                let mut color = if self.fog_of_war[y][x] {
                    *FOG_COLOR
                } else {
                    *DEFAULT_COLOR
                };

                if !self.fog_of_war[y][x] {
                    match (
                        self.energy[y][x],
                        self.minerals[y][x],
                        self.science_interests[y][x],
                        self.obstacles[y][x],
                    ) {
                        (true, false, false, false) => color = *ENERGY_COLOR,
                        (false, true, false, false) => color = *MINERALS_COLOR,
                        (false, false, true, false) => color = *SCIENCE_INTERESTS_COLOR,
                        (false, false, false, true) => color = *OBSTACLE_COLOR,
                        _ => color = *DEFAULT_COLOR,
                    }
                }

                let rect = graphics::Rect::new(
                    x as f32 * CELL_SIZE,
                    y as f32 * CELL_SIZE,
                    CELL_SIZE,
                    CELL_SIZE,
                );
                let mesh =
                    graphics::Mesh::new_rectangle(ctx, graphics::DrawMode::fill(), rect, color)?;
                graphics::draw(ctx, &mesh, graphics::DrawParam::default())?;
            }
        }

        if let Some(robot) = &self.robot_explorer {
            let station_rect = graphics::Rect::new(
                robot.station_x as f32 * CELL_SIZE,
                robot.station_y as f32 * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
            );
            let station_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                station_rect,
                *STATION_COLOR,
            )?;
            graphics::draw(ctx, &station_mesh, graphics::DrawParam::default())?;
        }

        if let Some(robot) = &self.robot_explorer {
            let robot_rect = graphics::Rect::new(
                robot.x as f32 * CELL_SIZE,
                robot.y as f32 * CELL_SIZE,
                CELL_SIZE,
                CELL_SIZE,
            );
            let robot_mesh = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                robot_rect,
                *ROBOT_COLOR,
            )?;
            graphics::draw(ctx, &robot_mesh, graphics::DrawParam::default())?;
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if timer::check_update_time(ctx, 1) {
            self.update_robot();
        }
        Ok(())
    }
}

fn main() -> GameResult {
    let cb = ContextBuilder::new("CosmoBots", "Alpha")
        .window_setup(conf::WindowSetup::default().title("CosmoBots"))
        .window_mode(conf::WindowMode::default().dimensions(
            (MAP_SIZE as f32 * CELL_SIZE) + 1.0,
            (MAP_SIZE as f32 * CELL_SIZE) + 1.0,
        ));

    let mut map = Map::new();

    // Place les ressources
    map.place_resources();

    // Génère les emplacements des ressources
    let resources: Vec<(usize, usize)> = map
        .energy
        .iter()
        .enumerate()
        .flat_map(|(y, row)| {
            row.iter()
                .enumerate()
                .filter(|(_, &val)| val)
                .map(move |(x, _)| (x, y))
        })
        .collect();

    // Place les obstacles en évitant les emplacements des ressources
    map.place_obstacles(&resources);

    let (ctx, event_loop) = &mut cb.build()?;
    event::run(ctx, event_loop, &mut map)
}
