use std::env;
use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::prelude::*;
use iyes_perf_ui::prelude::*;
use rand::prelude::*;

use crate::types::{Agent, ChunkCoordinates, Simulation};

mod types;

const CANVAS_WIDTH: f32 = 1440.;
const CANVAS_HEIGHT: f32 = 1440.;
const CHUNK_SIZE: f32 = 100.;

fn main() {
    let args: Vec<String> = env::args().collect();
    let entity_count: usize = args[1]
        .parse()
        .expect("Wrong args");

    App::new()
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(
                    Window {
                        mode: WindowMode::Fullscreen,
                        title: "Simulator".into(),
                        resolution: (CANVAS_WIDTH, CANVAS_HEIGHT).into(),
                        ..default()
                    }
                ),
                ..default()
            }))
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .add_plugins(PanCamPlugin)
        .insert_resource(Simulation {
            entity_count,
            canvas_w: CANVAS_WIDTH,
            canvas_h: CANVAS_HEIGHT,
            chunk_size: CHUNK_SIZE,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_plugins(ShapePlugin)
        .add_systems(Startup, (setup, setup_grid, setup_debug).chain())
        .add_systems(Update, (move_squares, color_squares))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(
        PanCam {
            grab_buttons: vec![MouseButton::Middle], // which buttons should drag the camera
            enabled: true, // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 1., // prevent the camera from zooming too far in
            max_scale: Some(40.), // prevent the camera from zooming too far out
            ..default()
        });
    commands.spawn((
        PerfUiRoot {
            display_labels: false,
            layout_horizontal: true,
            ..default()
        },
        PerfUiEntryFPSWorst::default(),
        PerfUiEntryFPS::default(),
    ));
}

fn setup_grid(
    mut commands: Commands,
    mut simul: ResMut<Simulation>,
) {
    let limits = simul.get_chunk_limits();
    for y in 0..limits.y {
        simul.chunks.push(vec![]);
        for x in 0..limits.x {
            let (r_x, r_y) = simul.get_global_coords(ChunkCoordinates::new(x, y));
            spawn_debug_square(&mut commands, r_x, r_y, simul.chunk_size);
            simul.chunks[y].push(vec![])
        }
    }
}

fn setup_debug(
    mut commands: Commands,
    mut simul: ResMut<Simulation>,
    squares: Query<(&mut Agent, &mut Transform)>,
    asset_server: Res<AssetServer>,
) {
    spawn_debug_entities(&mut commands, asset_server, &mut simul);
    squares.iter().for_each(|(_person, transform)| {
        let coords = simul.get_chunk_coords(transform.translation.x, transform.translation.y);
        simul.chunks[coords.x][coords.y].iter().for_each(|entity| {
            println!("{:?}: {}, {}", entity, coords.x, coords.y)
        });
    });
}

fn move_squares(
    mut simul: ResMut<Simulation>,
    time: Res<Time>,
    mut squares: Query<(Entity, &mut Agent, &mut Sprite, &mut Transform)>,
    q_window: Query<&Window>,
    // query to get camera transform
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    for (entity, mut agent, _sprite, mut transform) in squares.iter_mut() {
        // println!("{}", x);
        if agent.is_main {
            let (camera, camera_transform) = q_camera.single();

            let window = q_window.single();
            let mut coords: Vec2 = Default::default();
            
            if let Some(world_position) = window.cursor_position()
                .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
                .map(|ray| ray.origin.truncate())
            {
                coords = world_position;
            }
            
            transform.translation.x = coords.x;
            transform.translation.y = coords.y;
        } else {
            let mut rng = thread_rng();
            let x: f32 = rng.gen();
            let y: f32 = rng.gen();
            transform.translation.x += (x - 0.5) * time.delta().as_millis() as f32;
            transform.translation.y += (y - 0.5) * time.delta().as_millis() as f32;
            transform.translation.x = transform.translation.x.clamp(0. - CANVAS_WIDTH / 2., CANVAS_WIDTH / 2.);
            transform.translation.y = transform.translation.y.clamp(0. - CANVAS_HEIGHT / 2., CANVAS_HEIGHT / 2.);
        }

        let coords = simul.get_chunk_coords(transform.translation.x, transform.translation.y);
        // println!("{n_x}, {n_y}");
        if coords != agent.coords {
            simul.change_entity_sector(entity, agent.coords, coords);
        }
        agent.coords = coords;
    };
}

fn color_squares(
    // mut simul: ResMut<Simulation>,
    mut squares: Query<(Entity, &mut Agent, &mut Sprite)>,
) {
    let main_coords = squares.iter()
        .find(|(_entity, agent, _sprite)| { agent.is_main })
        .map(|(_entity, agent, _sprite)| {
            agent.coords
        }).unwrap();

    // Todo: iterate only chunk entities

    // let entities = simul.get_chunk_entities(main_coords);

    for (_entity, agent, mut sprite) in squares.iter_mut() {
        if agent.coords != main_coords {
            sprite.color = Color::rgb(0., 0., 1.);
            continue;
        }
        if agent.is_main {
            sprite.color = Color::rgb(1., 1., 0.);
        } else {
            sprite.color = Color::rgb(1., 0., 1.);
        }
    };
}

fn spawn_debug_entities(commands: &mut Commands, asset_server: Res<AssetServer>, simul: &mut ResMut<Simulation>) {
    let coords = simul.get_chunk_coords(-300., -300.);
    for i in 0..simul.entity_count {
        // let color = Color::hsl(360. * i as f32 / ENTITY_COUNT as f32, 0.95, 0.5);
        let entity = commands.spawn((
            SpriteBundle {
                texture: asset_server.load("sprite.png"),
                transform: Transform {
                    translation: Vec3 {
                        z: 10.,
                        ..default()
                    },
                    ..default()
                },
                ..default()
            },
            Agent {
                coords,
                is_main: i == simul.entity_count - 1,
            }
        )).id();
        simul.add_entity(coords, entity);
    }
}

fn spawn_debug_square(commands: &mut Commands, x: f32, y: f32, chunk_size: f32) {
    let shape = shapes::Rectangle {
        extents: Vec2::new(chunk_size, chunk_size),
        origin: RectangleOrigin::CustomCenter(Vec2::new(x, y)),
    };
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            ..default()
        },
        Stroke::new(Color::RED, 1.0),
    ));
}
