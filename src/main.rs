use ggez::graphics::Color;
use ggez::ContextBuilder;
use ggez::{conf, event, graphics, timer, Context, GameResult};
use lazy_static::lazy_static;
use noise::{Fbm, NoiseFn, Perlin};
use rand::seq::SliceRandom;
use rand::Rng;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::time::Duration;

mod map;
use crate::map::Map;

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

// Constantes de couleurs
const DEFAULT_COLOR: Color = Color::new(1.0, 1.0, 1.0, 1.0); // White
const OBSTACLE_COLOR: Color = Color::new(0.498, 0.498, 0.498, 1.0); // Gray
const ENERGY_COLOR: Color = Color::new(1.0, 1.0, 0.0, 1.0); // Yellow
const MINERALS_COLOR: Color = Color::new(0.0, 0.0, 1.0, 1.0); // Blue
const SCIENCE_INTERESTS_COLOR: Color = Color::new(0.0, 1.0, 0.0, 1.0); // Green
const ROBOT_EXPLORER_COLOR: Color = Color::new(1.0, 0.0, 0.0, 1.0); // Red
const ROBOT_EXTRACTOR_COLOR: Color = Color::new(1.0, 0.647, 0.0, 1.0); // Orange
const STATION_COLOR: Color = Color::new(0.0, 1.0, 1.0, 1.0); // Cyan
const FOG_COLOR: Color = Color::new(0.0, 0.0, 0.0, 1.0); // Black

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

struct RobotExplorer {
    x: usize,
    y: usize,
    station_x: usize,
    station_y: usize,
    founded_resource: bool,
    resource_position: Option<(usize, usize)>,
    waiting: bool, // Nouvel état
}

struct RobotExtractor {
    x: usize,
    y: usize,
    station_x: usize,
    station_y: usize,
    carrying_resource: bool,
    target_position: Option<(usize, usize)>,
    waiting: bool, // Nouvel état
}

impl RobotExtractor {
    fn new(station_x: usize, station_y: usize) -> Self {
        RobotExtractor {
            x: station_x,
            y: station_y,
            station_x,
            station_y,
            carrying_resource: false,
            target_position: None,
            waiting: false, // Initialisation
        }
    }

    fn move_towards(
        &mut self,
        target_x: usize,
        target_y: usize,
        obstacles: &[[bool; MAP_SIZE]; MAP_SIZE],
    ) -> Option<Vec<(usize, usize)>> {
        let start = (self.x, self.y);
        let goal = (target_x, target_y);

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

    fn collect_resource(&mut self) {
        self.carrying_resource = true;
        println!("Collected resource at ({}, {})", self.x, self.y);
    }
}

impl RobotExplorer {
    fn new(station_x: usize, station_y: usize) -> Self {
        RobotExplorer {
            x: station_x,
            y: station_y,
            station_x,
            station_y,
            founded_resource: false,
            resource_position: None,
            waiting: false, // Initialisation
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
                self.founded_resource = true;
                self.resource_position = Some((new_x, new_y)); // Ajouter cette ligne
                println!("Founded energy at ({}, {}).", new_x, new_y,);
            } else if map.minerals[new_y][new_x] {
                self.founded_resource = true;
                self.resource_position = Some((new_x, new_y)); // Ajouter cette ligne
                println!("Founded minerals at ({}, {}).", new_x, new_y,);
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

fn heuristic(x1: usize, y1: usize, x2: usize, y2: usize) -> usize {
    ((x2 as isize - x1 as isize).abs() + (y1 as isize - y2 as isize).abs()) as usize
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
