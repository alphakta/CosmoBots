use bevy::prelude::*;


fn main() {
    App::new()
    .add_plugins(DefaultPlugins)
    .add_systems(Startup, setup)
    .insert_resource(ClearColor(Color::rgb(0.95, 0.62, 0.)))
    .run();
 }
 
 #[derive(Component)]
enum Robot {
    Soil,
    Water,
    Rock
}

#[derive(Component)]
struct Name(String);


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
    commands.spawn((Robot::Soil, Name("Soil Robot".to_string())));
    commands.spawn((Robot::Water, Name("Water Robot".to_string())));
    commands.spawn((Robot::Rock, Name("Rock Robot".to_string())));
}