use bevy::prelude::*;
// use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
// use bevy::render::camera;
use std::collections::HashMap;
use rand::Rng;


fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(RobotMap::default())
    // .add_systems(Startup ,add_map_elements)
    .add_systems(Startup, setup)
    .add_systems(Startup, add_robots)
    .add_systems(Update , roaming_movement_system)
    .insert_resource(ClearColor(Color::rgb(0.95, 0.62, 0.)))
    .run();
 }


 #[derive(Default , Resource)]
pub struct RobotMap {
    pub robots: HashMap<RobotType, Vec<Entity>>,
}


#[derive(Component)]
pub struct RobotBundle {
    pub robot : Robot,
    pub transform: Transform,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum RobotType {
    Soil,
    Water,
    Rock,
}

#[derive(Component)]
pub struct Robot {
    pub name : String,
    pub durability : i32,
    pub autonomy : i32,
    pub speciality : RobotType,
    pub space : i32,
} 

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn create_robot(commands: &mut Commands, robot_type: RobotType) -> Entity {
    let color = match robot_type {
        RobotType::Soil => Color::rgb(0.3, 0.4, 0.6),
        RobotType::Water => Color::rgb(0.0, 0.0, 1.0),
        RobotType::Rock => Color::rgb(0.5, 0.5, 0.5),
    };

    let name = match robot_type {
        RobotType::Soil => "Soil Robot",
        RobotType::Water => "Water Robot",
        RobotType::Rock => "Rock Robot",
    }.to_string();

    let spawn_point = match robot_type {
        RobotType::Soil => Transform::from_xyz(-10.0, 0.0, 0.0),
        RobotType::Water =>  Transform::from_xyz(10.0, 0.0, 0.0),
        RobotType::Rock =>  Transform::from_xyz(0.0, 0.0, 0.0),
    };

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::new(10.0, 10.0)),
            ..default()
        },
        transform : spawn_point,
        ..default()
    })
    .insert(Robot {
        name,
        durability: 10,
        autonomy: 100,
        speciality: robot_type,
        space: 100,
    })
    .insert(Movement::new(2.0))
    .id()
}

fn add_robots(mut commands: Commands, mut robot_map: ResMut<RobotMap>) {
    let soil_robot = create_robot(&mut commands, RobotType::Soil);
    robot_map.robots.entry(RobotType::Soil).or_insert_with(Vec::new).push(soil_robot);

    let water_robot = create_robot(&mut commands, RobotType::Water);
    robot_map.robots.entry(RobotType::Water).or_insert_with(Vec::new).push(water_robot);

    let rock_robot = create_robot(&mut commands, RobotType::Rock);
    robot_map.robots.entry(RobotType::Rock).or_insert_with(Vec::new).push(rock_robot);

}

// fn add_map_elements(mut commands: Commands, asset_server: Res<AssetServer>) {
//     let element_texture_handle = asset_server.load("textures/256_Grass 02 Blades.png");

//     commands.spawn(SpriteBundle {
//         texture: element_texture_handle,
//         transform: Transform::from_xyz(0.0, 0.0, 0.0), 
//         ..default()
//     });
// }

#[derive(Component)]
struct Movement {
    direction: Vec3, // Direction actuelle du mouvement
    timer: Timer,    // Timer pour changer de direction
}

impl Movement {
    fn new(duration: f32) -> Self {
        Movement {
            direction: Vec3::ZERO,
            timer: Timer::from_seconds(duration , TimerMode::Once),
        }
    }
}

fn roaming_movement_system(
    time: Res<Time>,
    mut query: Query<(&mut Movement, &mut Transform)>,
) {
    let mut rng = rand::thread_rng();

    for (mut movement, mut transform) in query.iter_mut() {
        // Mise à jour du timer
        movement.timer.tick(time.delta());

        if movement.timer.just_finished() {
            // Choisir une nouvelle direction aléatoire
            let dx = rng.gen_range(-1.0..1.0);
            let dy = rng.gen_range(-1.0..1.0);
            movement.direction = Vec3::new(dx, dy, 0.0).normalize() * 4.0; // Normaliser pour garder une vitesse constante
        }

        // Appliquer le mouvement
        transform.translation += movement.direction * time.delta_seconds();
    }
}