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
    pub global_transform: GlobalTransform,
    pub texture: Handle<Image>,
    pub visibility: Visibility,
    pub inherited_visibility: InheritedVisibility,
    pub view_visibility: ViewVisibility,
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

    commands.spawn(Robot{
        color : Color::rgb(0.5, 0.5, 0.5),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        speciality : RobotType::Soil,
        name : "Soil Robot".to_string(),
        durability : 10,
        autonomy : 100,
        flip_x : false,
        flip_y : false
    });
    commands.spawn(RobotBundle {
        robot : Robot{
        color : Color::rgb(0.5, 0.5, 0.5),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        speciality : RobotType::Soil,
        name : "Soil Robot".to_string(),
        durability : 10,
        autonomy : 100,
        flip_x : false,
        flip_y : false
        },
        transform : Transform::from_xyz(0.0, 0.0, 0.0),
       ..default()
    });

    commands.spawn(Robot{
        color : Color::rgb(0.5, 0.5, 0.5),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        speciality : RobotType::Soil,
        name : "Water Robot".to_string(),
        durability : 30,
        autonomy : 40,
        flip_x : false,
        flip_y : false
    });
    commands.spawn(Robot{
        color : Color::rgb(0.5, 0.5, 0.5),
        custom_size: Some(Vec2::new(10.0, 10.0)),
        speciality : RobotType::Soil,
        name : "Rock Robot".to_string(),
        durability : 10,
        autonomy : 300,
        flip_x : false,
        flip_y : false
    });

}



// fn check_mouse_clicks(
//     mouse_button_input: Res<Input<MouseButton>>,
//     mut cursor_moved_events: EventReader<CursorMoved>,
// ) {
//     // Vérifier si le bouton gauche de la souris a été pressé
//     if mouse_button_input.just_pressed(MouseButton::Left) {
//         println!("Mouse button left clicked!");

//     }

//     // Exemple pour détecter le mouvement de la souris
//     for event in cursor_moved_events.iter() {
//         println!("Cursor moved to: {:?}", event.position);
//     }
// }