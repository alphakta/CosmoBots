use rand::prelude::*;
use std::collections::{BinaryHeap, HashMap};

use crate::{heuristic, Map, Node, DIRECTIONS, MAP_SIZE};

pub struct RobotExplorer {
    pub x: usize,
    pub y: usize,
    pub station_x: usize,
    pub station_y: usize,
    pub founded_resource: bool,
    pub resource_position: Option<(usize, usize)>,
    pub waiting: bool,
}

pub struct RobotExtractor {
    pub x: usize,
    pub y: usize,
    pub station_x: usize,
    pub station_y: usize,
    pub carrying_resource: bool,
    pub target_position: Option<(usize, usize)>,
    pub waiting: bool,
}

impl RobotExtractor {
    pub fn new(station_x: usize, station_y: usize) -> Self {
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

    pub fn move_towards(
        &mut self,
        target_x: usize,
        target_y: usize,
        obstacles: &[[bool; MAP_SIZE]; MAP_SIZE],
        fog_of_war: &[[bool; MAP_SIZE]; MAP_SIZE],
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
                if obstacles[neighbor_y][neighbor_x] || fog_of_war[neighbor_y][neighbor_x] {
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

    pub fn collect_resource(&mut self) {
        self.carrying_resource = true;
        println!("Collected resource at ({}, {})", self.x, self.y);
    }
}

impl RobotExplorer {
    pub fn new(station_x: usize, station_y: usize) -> Self {
        RobotExplorer {
            x: station_x,
            y: station_y,
            station_x,
            station_y,
            founded_resource: false,
            resource_position: None,
            waiting: false,
        }
    }

    pub fn move_random(&mut self, rng: &mut impl Rng, map: &mut Map) {
        let mut possible_moves = vec![];

        for &(dx, dy) in &DIRECTIONS {
            let new_x = (self.x as isize + dx).max(0).min(MAP_SIZE as isize - 1) as usize;
            let new_y = (self.y as isize + dy).max(0).min(MAP_SIZE as isize - 1) as usize;

            // Ne pas ajouter la station comme un mouvement possible
            if (new_x, new_y) == (self.station_x, self.station_y) {
                continue;
            }

            if !map.obstacles[new_y][new_x] && !map.explored[new_y][new_x] {
                possible_moves.push((new_x, new_y));
            }
        }

        if possible_moves.is_empty() {
            for &(dx, dy) in &DIRECTIONS {
                let new_x = (self.x as isize + dx).max(0).min(MAP_SIZE as isize - 1) as usize;
                let new_y = (self.y as isize + dy).max(0).min(MAP_SIZE as isize - 1) as usize;

                // Ne pas ajouter la station comme un mouvement possible
                if (new_x, new_y) == (self.station_x, self.station_y) {
                    continue;
                }

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
                self.resource_position = Some((new_x, new_y));
                println!("Founded energy at ({}, {}).", new_x, new_y);
            } else if map.minerals[new_y][new_x] {
                self.founded_resource = true;
                self.resource_position = Some((new_x, new_y));
                println!("Founded minerals at ({}, {}).", new_x, new_y);
            }
        }
    }

    pub fn return_to_station(
        &mut self,
        obstacles: &[[bool; MAP_SIZE]; MAP_SIZE],
        fog_of_war: &[[bool; MAP_SIZE]; MAP_SIZE],
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
                if obstacles[neighbor_y][neighbor_x] || fog_of_war[neighbor_y][neighbor_x] {
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
