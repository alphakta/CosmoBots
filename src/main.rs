use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
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
    .add_systems(Startup, create_lake)
    .add_systems(Update , roaming_movement_system)

    .insert_resource(ClearColor(Color::rgb(0.95, 0.62, 0.)))
    .run();
 }


 #[derive(Default , Resource)]
pub struct RobotMap {
    pub robots: HashMap<MaterialType, Vec<Entity>>,
}


#[derive(Component)]
pub struct RobotBundle {
    pub robot : Robot,
    pub transform: Transform,
}

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum MaterialType {
    Soil,
    Water,
    Rock
}

#[derive(Component)]
pub struct Robot {
    pub name : String,
    pub durability : i32,
    pub autonomy : i32,
    pub speciality : MaterialType,
    pub space : i32,
} 

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn create_robot(commands: &mut Commands, robot_type: MaterialType) -> Entity {
    let color = match robot_type {
        MaterialType::Soil => Color::rgb(0.3, 0.4, 0.6),
        MaterialType::Water => Color::rgb(0.0, 0.0, 1.0),
        MaterialType::Rock => Color::rgb(0.5, 0.5, 0.5),
    };

    let name = match robot_type {
        MaterialType::Soil => "Soil Robot",
        MaterialType::Water => "Water Robot",
        MaterialType::Rock => "Rock Robot",
    }.to_string();

    let spawn_point = match robot_type {
        MaterialType::Soil => Transform::from_xyz(-10.0, 0.0, 0.0),
        MaterialType::Water => Transform::from_xyz(10.0, 0.0, 0.0),
        MaterialType::Rock => Transform::from_xyz(0.0, 0.0, 0.0),
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
    let soil_robot = create_robot(&mut commands, MaterialType::Soil);
    robot_map.robots.entry(MaterialType::Soil).or_insert_with(Vec::new).push(soil_robot);

    let water_robot = create_robot(&mut commands, MaterialType::Water);
    robot_map.robots.entry(MaterialType::Water).or_insert_with(Vec::new).push(water_robot);

    let rock_robot = create_robot(&mut commands, MaterialType::Rock);
    robot_map.robots.entry(MaterialType::Rock).or_insert_with(Vec::new).push(rock_robot);

}

//-------- TEST ------------------------------------
const X_EXTENT: f32 = 600.;


fn create_lake(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>
){
    let lake_shape : Mesh2dHandle =  Mesh2dHandle(meshes.add(Ellipse::new(25.0, 50.0)));
    let lake_color : Color = Color::rgb(0.0,1.0 , 1.0);
    let mut rng = rand::thread_rng();
    let random_pos_x : f32 = rng.gen_range(-200.0 .. 200.0);
    let random_pos_y : f32 = rng.gen_range(-200.0 .. 200.0);
    commands.spawn(MaterialMesh2dBundle {
        mesh: lake_shape,
            material: materials.add(lake_color),
            transform: Transform::from_xyz(
                random_pos_x,
                random_pos_y,
                0.0,
            ),
            ..default()
    });
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
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

    for (mut movement, mut transform) in query.iter_mut() {
        // Mise à jour du timer
        movement.timer.tick(time.delta());

        if movement.timer.just_finished() {
            // Choisir une nouvelle direction aléatoire
            let dx: f32 = rng.gen_range(-1.0..1.0);
            let dy: f32 = rng.gen_range(-1.0..1.0);
            movement.direction = Vec3::new(dx, dy, 0.0).normalize() * 4.0; // Normaliser pour garder une vitesse constante
        }

        // Appliquer le mouvement
        transform.translation += movement.direction * time.delta_seconds();
    }
}