use bevy::prelude::*;
// use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
use bevy::render::camera;


fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .add_systems(Startup, add_robots)
    .insert_resource(ClearColor(Color::rgb(0.95, 0.62, 0.)))
    .run();
 }

#[derive(Component)]
pub struct RobotBundle {
    pub robot : Robot,
    pub transform: Transform,
}

enum RobotType {
    Soil,
    Water,
    Rock
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

fn add_robots(mut commands: Commands) {

    let water_robot_sprite = Sprite {
        color: Color::rgb(0.0, 0.0, 1.0),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        ..default()
    };

    let soil_robot_sprite = Sprite {
        color: Color::rgb(0.3, 0.4, 0.6),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        ..default()
    };

    let rock_robot_sprite = Sprite {
        color: Color::rgb(0.5, 0.5, 0.5),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        ..default()
    };

    commands.spawn(SpriteBundle {
        sprite: soil_robot_sprite,
        transform: Transform::from_xyz(-10.0, -10.0, 0.0),
        ..default()
    })
    .insert(Robot {
        name: "Soil Robot".to_string(),
        durability: 10,
        autonomy: 100,
        speciality: RobotType::Soil,
        space : 100
    });

    commands.spawn(SpriteBundle {
        sprite: water_robot_sprite,
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    })
    .insert(Robot {
        name: "Water Robot".to_string(),
        durability: 10,
        autonomy: 100,
        speciality: RobotType::Water,
        space : 100
    });
    commands.spawn(SpriteBundle {
        sprite: rock_robot_sprite,
        transform: Transform::from_xyz(10.0, 10.0, 0.0),
        ..default()
    })
    .insert(Robot {
        name: "Rock Robot".to_string(),
        durability: 10,
        autonomy: 100,
        speciality: RobotType::Rock,
        space : 100
    });

    
}