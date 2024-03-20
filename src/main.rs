use bevy::prelude::*;
// use bevy::input::mouse::{MouseButtonInput, MouseMotion, MouseWheel};
// use bevy::render::camera;
use bevy::utils::HashMap;


fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .insert_resource(RobotMap::default())
    .add_systems(Startup, setup)
    .add_systems(Startup, add_robots)
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

fn create_robot(commands: &mut Commands, robot_type: RobotType) {
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

    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color,
            custom_size: Some(Vec2::new(10.0, 10.0)),
            ..default()
        },
        ..default()
    })
    .insert(Robot {
        name,
        durability: 10,
        autonomy: 100,
        speciality: robot_type,
        space: 100,
    });
}

fn add_robots(mut commands: Commands) {
    create_robot(&mut commands, RobotType::Soil);
    create_robot(&mut commands, RobotType::Water);
    create_robot(&mut commands, RobotType::Rock);
}
fn robot_mouvment(){

}