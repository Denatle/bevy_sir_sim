use std::env;

use bevy::window::WindowMode;
use bevy::{prelude::*, window::WindowResolution};
use bevy_prototype_lyon::prelude::*;
use iyes_perf_ui::prelude::*;
use rand::prelude::*;

use chunk_lib::{ChunkCoordinates, Chunkable, ChunkableBundle, Chunking, Simulation};

use crate::types::Agent;

mod types;

const CANVAS_WIDTH: f32 = 1920.;
const CANVAS_HEIGHT: f32 = 1080.;
const CHUNK_WIDTH: f32 = 64.;
const CHUNK_HEIGHT: f32 = 27.;

fn main() {
    let args: Vec<String> = env::args().collect();
    let entity_count: usize = args[1].parse().expect("Wrong args");

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    mode: WindowMode::Windowed,
                    title: "Bad Apple!".into(),
                    resolution: WindowResolution::new(CANVAS_WIDTH, CANVAS_HEIGHT)
                        .with_scale_factor_override(1.),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(Chunking {
            entity_count,
            canvas_w: CANVAS_WIDTH,
            canvas_h: CANVAS_HEIGHT,
            chunk_w: CHUNK_WIDTH,
            chunk_h: CHUNK_HEIGHT,
        })
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_plugins(ShapePlugin)
        .add_systems(Startup, (setup, setup_debug).chain())
        .add_systems(Update, (change_destination, move_agents))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
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

fn setup_debug(
    mut commands: Commands,
    mut simul: ResMut<Simulation>,
    asset_server: Res<AssetServer>,
) {
    // let limits = simul.get_chunk_limits();
    // for y in 0..limits.y {
    //     for x in 0..limits.x {
    //         let (r_x, r_y) = simul.get_global_coords(ChunkCoordinates::new(x, y));
    //         spawn_debug_square(&mut commands, r_x, r_y, simul.chunk_w, simul.chunk_h);
    //     }
    // }
    spawn_agents(&mut commands, &asset_server, &mut simul);
}

fn move_agents(time: Res<Time>, mut squares: Query<(&mut Agent, &mut Transform)>) {
    for (agent, mut transform) in squares.iter_mut() {
        let old_translation = transform.translation;
        transform.translation +=
            (agent.destination - old_translation) * agent.speed * time.delta().as_secs_f32();
    }
}

fn change_destination(
    simul: ResMut<Simulation>,
    mut agent_query: Query<(&mut Agent, &Chunkable, &Transform)>,
) {
    for (mut agent, chunkable, transform) in agent_query.iter_mut() {
        if agent.destination.round() != transform.translation.round() {
            continue;
        }
        let x_dev = chunkable.coords.x as f32 + ((random::<f32>() - 0.5) * 2.).round_ties_even();
        let y_dev = chunkable.coords.y as f32 + ((random::<f32>() - 0.5) * 2.).round_ties_even();

        let n_chunk = ChunkCoordinates::new(x_dev as usize, y_dev as usize);

        let (n_x, n_y) = simul.get_global_coords(n_chunk);

        let r_x = ((random::<f32>() - 0.5) * simul.chunk_w + n_x)
            .clamp(0. - simul.canvas_w / 2.31, simul.canvas_w / 2.31);
        let r_y = ((random::<f32>() - 0.5) * simul.chunk_h + n_y)
            .clamp(0. - simul.canvas_w / 2.31, simul.canvas_w / 2.31);

        // println!("{}, {}", x_dev, y_dev);
        let destination = Vec3::new(r_x, r_y, 0.);
        agent.destination = destination;
    }
}

fn spawn_agents(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    simul: &mut ResMut<Simulation>,
) {
    let coords = simul.get_chunk_coords(0., 0.);
    for _ in 0..simul.entity_count {
        let x: f32 = (random::<f32>() - 0.5) * (simul.canvas_w / 2.);
        let y: f32 = (random::<f32>() - 0.5) * (simul.canvas_h / 2.);
        let destination = Vec3::new(x, y, 0.);
        let entity = commands
            .spawn((
                ChunkableBundle {
                    chunkable: Chunkable { coords },
                },
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
                    speed: 3.,
                    destination,
                },
            ))
            .id();
        simul.add_entity(coords, entity);
    }
}

fn spawn_debug_square(commands: &mut Commands, x: f32, y: f32, chunk_w: f32, chunk_h: f32) {
    let shape = shapes::Rectangle {
        extents: Vec2::new(chunk_w, chunk_h),
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
