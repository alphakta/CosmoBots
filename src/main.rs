use bevy::prelude::*;
// use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
// use bevy::render::camera;


fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, (setup , add_robots ))
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
    pub color: Color,
    pub name : String,
    pub durability : i32,
    pub autonomy : i32,
    pub speciality : RobotType,
    pub custom_size: Option<Vec2>,
    pub space : i32,
    pub flip_x: bool,
    pub flip_y: bool,
} 

fn setup(mut commands: Commands) {

    commands.spawn(Camera2dBundle::default());

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(0.5, 0.5, 0.5), 
            custom_size: Some(Vec2::new(10.0, 10.0)), 
            ..default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0), 
        ..default()
    });
}

fn add_robots(mut commands: Commands) {

    commands.spawn(RobotBundle {
        robot : Robot{
        color : Color::rgb(0.5, 0.5, 0.5),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        speciality : RobotType::Soil,
        name : "Soil Robot".to_string(),
        durability : 10,
        space: 100,
        autonomy : 100,
        flip_x : false,
        flip_y : false
        },
        transform : Transform::from_xyz(0.0, 0.0, 0.0),
    });
    commands.spawn(RobotBundle {
        robot : Robot{
        color : Color::rgb(0.5, 0.5, 0.5),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        speciality : RobotType::Water,
        name : "Water Robot".to_string(),
        durability : 10,
        space: 100,
        autonomy : 100,
        flip_x : false,
        flip_y : false
        },
        transform : Transform::from_xyz(-10.0, -10.0, 0.0),
    });
    commands.spawn(RobotBundle {
        robot : Robot{
        color : Color::rgb(0.5, 0.5, 0.5),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        speciality : RobotType::Rock,
        name : "Rock Robot".to_string(),
        durability : 10,
        space: 100,
        autonomy : 100,
        flip_x : false,
        flip_y : false
        },
        transform : Transform::from_xyz(10.0, 10.0, 0.0),
    });

}