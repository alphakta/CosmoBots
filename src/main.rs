use ggez::audio;
use ggez::audio::SoundSource;
use ggez::conf;
use ggez::event::{self, EventHandler};
use ggez::glam::*;
use ggez::graphics::{self, Color};
use ggez::input::keyboard::KeyCode;
use ggez::timer;
use ggez::{Context, ContextBuilder, GameResult};

use ggez::input::keyboard::KeyInput;
use oorandom::Rand32;
use std::env;
use std::path;

type Point2 = Vec2;
type Vector2 = Vec2;

enum ActorType {
    Robot,
    Obstacle,
    Minerals,
    Energy,
    ResearchCentre,
}
struct Actor {
    tag: ActorType,
    pos: Point2,
    velocity: Vector2,
    life: f32,
}

const RESOURCES_LIFE: f32 = 1.0;
const ROBOT_LIFE: f32 = 1.0;

fn vec_from_angle(angle: f32) -> Vector2 {
    let vx = angle.sin();
    let vy = angle.cos();
    Vector2::new(vx, vy)
}

fn create_robot() -> Actor {
    Actor {
        tag: ActorType::Robot,
        pos: Point2::ZERO,
        velocity: Vector2::ZERO,
        life: ROBOT_LIFE,
    }
}

fn create_minerals_resources() -> Actor {
    Actor {
        tag: ActorType::Minerals,
        pos: Point2::ZERO,
        velocity: Vector2::ZERO,
        life: RESOURCES_LIFE,
    }
}

fn create_energy_resources() -> Actor {
    Actor {
        tag: ActorType::Energy,
        pos: Point2::ZERO,
        velocity: Vector2::ZERO,
        life: RESOURCES_LIFE,
    }
}

fn create_research_centre_resources() -> Actor {
    Actor {
        tag: ActorType::ResearchCentre,
        pos: Point2::ZERO,
        velocity: Vector2::ZERO,
        life: RESOURCES_LIFE,
    }
}

fn create_obstacle() -> Actor {
    Actor {
        tag: ActorType::Obstacle,
        pos: Point2::ZERO,
        velocity: Vector2::ZERO,
        life: 1.0,
    }
}

fn world_to_screen_coords(screen_width: f32, screen_height: f32, point: Point2) -> Point2 {
    let x = point.x + screen_width / 2.0;
    let y = screen_height - (point.y + screen_height / 2.0);
    Point2::new(x, y)
}

/// Crée le nombre donné de ressources.
/// Assurez-vous qu'aucune d'entre elles ne se trouve dans la
/// zone d'exclusion donnée (nominalement le joueur)
/// Notez que cela pourrait créer des ressources en dehors de
/// des limites du terrain de jeu, donc cela devrait être
/// appelé avant que `wrap_actor_position()` ne se produise.
fn create_resources(
    rng: &mut Rand32,
    num: i32,
    exclusion: Point2,
    min_distance: f32,
    max_distance: f32,
) -> Vec<Actor> {
    assert!(max_distance > min_distance);
    let new_resource = |_| {
        let mut resource = match rng.rand_range(0..3) {
            0 => create_minerals_resources(),
            1 => create_energy_resources(),
            _ => create_research_centre_resources(),
        };
        let angle = rng.rand_float() * 2.0 * std::f32::consts::PI;
        let distance = rng.rand_float() * (max_distance - min_distance) + min_distance;
        resource.pos = exclusion + vec_from_angle(angle) * distance;
        resource
    };
    (0..num).map(new_resource).collect()
}

fn create_obstacles(
    rng: &mut Rand32,
    num: i32,
    exclusion: Point2,
    min_radius: f32,
    max_radius: f32,
) -> Vec<Actor> {
    assert!(max_radius > min_radius);
    let new_obstacle = |_| {
        let mut obstacle = create_obstacle();
        let o_angle = rng.rand_float() * 2.0 * std::f32::consts::PI;
        let o_distance = rng.rand_float() * (max_radius - min_radius) + min_radius;
        obstacle.pos = exclusion + vec_from_angle(o_angle) * o_distance;
        obstacle
    };
    (0..num).map(new_obstacle).collect()
}

fn create_robots(
    rng: &mut Rand32,
    num: i32,
    exclusion: Point2,
    min_radius: f32,
    max_radius: f32,
) -> Vec<Actor> {
    assert!(max_radius > min_radius);
    let new_robot = |_| {
        let mut robot = create_robot();
        let r_angle = rng.rand_float() * 2.0 * std::f32::consts::PI;
        let r_distance = rng.rand_float() * (max_radius - min_radius) + min_radius;
        robot.pos = exclusion + vec_from_angle(r_angle) * r_distance;
        robot
    };
    (0..num).map(new_robot).collect()
}

fn update_robot_position(
    robot: &mut Actor,
    obstacles: &[Actor],
    resources: &[Actor],
    dt: f32,
    rng: &mut Rand32,
    screen_width: f32,
    screen_height: f32,
) {
    const MAX_ROBOT_SPEED: f32 = 100.0; // Vitesse maximale du robot

    // Générer une nouvelle direction de déplacement aléatoire à chaque mise à jour
    let new_direction = random_direction(rng);

    // Appliquer la direction et la vitesse pour mettre à jour la position du robot
    robot.velocity = new_direction * MAX_ROBOT_SPEED;
    let dv = robot.velocity * dt;
    let new_pos = robot.pos + dv * 2.0;

    // Vérifier les collisions avec les obstacles et ajuster la position si nécessaire
    for obstacle in obstacles {
        if collides_with(robot, obstacle) {
            // Si une collision est détectée avec un obstacle, arrêtez le mouvement du robot
            robot.velocity = Vector2::ZERO;
            break;
        }
    }

    // Vérifier les collisions avec les ressources et afficher un message si nécessaire
    for resource in resources {
        if collides_with(robot, resource) {
            println!("Robot collided with a resource!");
            break;
        }
    }

    // Vérifier si la nouvelle position dépasse les limites de l'écran
    let robot_radius = get_robot_radius();
    let min_x = robot_radius;
    let max_x = screen_width - robot_radius;
    let min_y = robot_radius;
    let max_y = screen_height - robot_radius;

    // Ajuster la position du robot pour rester à l'intérieur des limites de l'écran
    robot.pos.x = new_pos.x.clamp(min_x, max_x);
    robot.pos.y = new_pos.y.clamp(min_y, max_y);
}

fn get_robot_radius() -> f32 {
    // Replace with the actual robot radius
    // For example, if the robot radius is 10.0, you can use:
    10.0
}

fn get_obstacle_radius() -> f32 {
    // Replace with the actual robot radius
    // For example, if the robot radius is 10.0, you can use:
    20.0
}

fn collides_with(robot: &Actor, obstacle: &Actor) -> bool {
    let robot_radius = get_robot_radius();
    let obstacle_radius = get_obstacle_radius(); // Replace with the actual obstacle radius

    let distance = (robot.pos - obstacle.pos).length();
    distance < robot_radius + obstacle_radius
}

fn random_direction(rng: &mut Rand32) -> Vector2 {
    let angle = rng.rand_float() * 4.0 * std::f32::consts::FRAC_PI_2; // Limit angles to 0, π/2, π, 3π/2
    vec_from_angle(angle)
}

// Constantes du jeu, assets et paramètres
struct Assets {
    robot_image: graphics::Image,
    minerals_image: graphics::Image,
    energy_image: graphics::Image,
    researchcentre_image: graphics::Image,
    obstacle_image: graphics::Image,
}

impl Assets {
    fn new(ctx: &mut Context) -> GameResult<Assets> {
        let robot_img = graphics::Image::from_path(ctx, "/robot.png")?;
        let minerals_img = graphics::Image::from_path(ctx, "/blue.png")?;
        let energy_img = graphics::Image::from_path(ctx, "/yellow.png")?;
        let research_centre_img = graphics::Image::from_path(ctx, "/grey.png")?;
        let obstacle_img = graphics::Image::from_path(ctx, "/tile_grey.png")?;

        Ok(Assets {
            robot_image: robot_img,
            minerals_image: minerals_img,
            energy_image: energy_img,
            researchcentre_image: research_centre_img,
            obstacle_image: obstacle_img,
        })
    }

    fn actor_image(&self, actor: &Actor) -> &graphics::Image {
        match actor.tag {
            ActorType::Robot => &self.robot_image,
            ActorType::Minerals => &self.minerals_image,
            ActorType::Energy => &self.energy_image,
            ActorType::ResearchCentre => &self.researchcentre_image,
            ActorType::Obstacle => &self.obstacle_image,
        }
    }
}

// Structure du jeu et implémentation de l'interface EventHandler
struct MainState {
    screen: graphics::ScreenImage,
    robot: Vec<Actor>,
    resources: Vec<Actor>,
    obstacles: Vec<Actor>,
    assets: Assets,
    screen_width: f32,
    screen_height: f32,
    rng: Rand32,
}

fn print_instructions() {
    println!();
    println!("Welcome to CosmoBots!");
    println!();
    println!();
}

fn draw_actor(
    assets: &mut Assets,
    canvas: &mut graphics::Canvas,
    actor: &Actor,
    world_coords: (f32, f32),
) {
    let (screen_w, screen_h) = world_coords;
    let pos = world_to_screen_coords(screen_w, screen_h, actor.pos);
    let image = assets.actor_image(actor);
    let drawparams = graphics::DrawParam::new()
        .dest(pos)
        .offset(Point2::new(0.5, 0.5));
    canvas.draw(image, drawparams);
}

impl MainState {
    fn new(ctx: &mut Context) -> GameResult<MainState> {
        println!("Game resource path: {:?}", ctx.fs);

        print_instructions();

        // Seed our RNG
        let mut seed: [u8; 8] = [0; 8];
        getrandom::getrandom(&mut seed[..]).expect("Could not create RNG seed");
        let mut rng = Rand32::new(u64::from_ne_bytes(seed));

        let assets = Assets::new(ctx)?;

        let num_robots = 5; // Nombre de robots à initialiser
        let exclusion_zone = Point2::new(0.0, 0.0); // Zone d'exclusion où les robots ne peuvent pas apparaître

        let robots = create_robots(&mut rng, num_robots, exclusion_zone, 100.0, 250.0);

        let resources = create_resources(&mut rng, 15, robots[0].pos, 100.0, 250.0);
        let obstacles = create_obstacles(&mut rng, 30, robots[0].pos, 100.0, 250.0);

        let (width, height) = ctx.gfx.drawable_size();
        let screen =
            graphics::ScreenImage::new(ctx, graphics::ImageFormat::Rgba8UnormSrgb, 1., 1., 1);
        println!("Screen dimensions: {} x {}", width, height);

        let s = MainState {
            screen,
            robot: robots,
            resources,
            obstacles,
            assets,
            screen_width: width,
            screen_height: height,
            rng,
        };

        Ok(s)
    }
}

impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        const DESIRED_FPS: u32 = 60;
        while ctx.time.check_update_time(DESIRED_FPS) {
            // Update the game state here
            for robot in &mut self.robot {
                update_robot_position(
                    robot,
                    &self.obstacles,
                    &self.resources,
                    1.0 / DESIRED_FPS as f32,
                    &mut self.rng,
                    self.screen_width,
                    self.screen_height,
                );
            }
        }
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_screen_image(ctx, &mut self.screen, Color::BLACK);

        // Loop over all objects drawing them...
        {
            let assets = &mut self.assets;
            let coords = (self.screen_width, self.screen_height);

            for r in &self.robot {
                draw_actor(assets, &mut canvas, r, coords);
            }

            for res in &self.resources {
                draw_actor(assets, &mut canvas, res, coords);
            }

            for o in &self.obstacles {
                draw_actor(assets, &mut canvas, o, coords);
            }
        }

        canvas.finish(ctx)?;
        ctx.gfx.present(&self.screen.image(ctx))?;

        timer::yield_now();
        Ok(())
    }
}

pub fn main() -> GameResult {
    // We add the CARGO_MANIFEST_DIR/resources to the resource paths
    // so that ggez will look in our cargo project directory for files.
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ContextBuilder::new("CosmoBots", "Alpha")
        .window_setup(conf::WindowSetup::default().title("CosmoBots!"))
        .window_mode(conf::WindowMode::default().dimensions(800.0, 600.0))
        .add_resource_path(resource_dir);

    let (mut ctx, events_loop) = cb.build()?;

    let game = MainState::new(&mut ctx)?;
    event::run(ctx, events_loop, game)
}
