use ggez::graphics::Color;
use ggez::ContextBuilder;
use ggez::{conf, event, GameResult};
use std::cmp::Ordering;

mod map;
mod robot;

use crate::map::Map;
use crate::robot::{RobotExplorer, RobotExtractor};

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
// Implémentation de la structure Node
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
// Implémentation des traits Ord et PartialOrd pour la structure Node
impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}
// Implémentation du trait PartialOrd pour la structure Node
impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Fonction de calcul de l'heuristique
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
