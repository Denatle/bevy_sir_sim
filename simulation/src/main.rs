use std::time::Duration;

use bevy::window::WindowMode;
use bevy::{prelude::*, window::WindowResolution};
use bevy_easings::*;
use bevy_prototype_lyon::prelude::*;
use iyes_perf_ui::prelude::*;
use rand::prelude::*;

use chunk_lib::{ChunkCoordinates, Chunkable, ChunkableBundle, Chunking, Simulation};

use crate::types::Agent;

mod types;

const AGENT_COUNT: usize = 1000;
const CANVAS_WIDTH: f32 = 1920.;
const CANVAS_HEIGHT: f32 = 1080.;
const CHUNK_WIDTH: f32 = 128.;
const CHUNK_HEIGHT: f32 = 54.;
const TRAVEL_DURATION: Duration = Duration::from_millis(300);
const EASING: EaseFunction = EaseFunction::QuarticInOut;

fn main() {
    // let args: Vec<String> = env::args().collect();
    // let entity_count: usize = args[1].parse().expect("Wrong args");

    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    present_mode: bevy::window::PresentMode::AutoNoVsync,
                    mode: WindowMode::Windowed,
                    title: "Bad Apple!!".into(),
                    resolution: WindowResolution::new(CANVAS_WIDTH, CANVAS_HEIGHT)
                        .with_scale_factor_override(1.),
                    ..default()
                }),
                ..default()
            }),
        )
        .add_plugins(Chunking {
            canvas_w: CANVAS_WIDTH,
            canvas_h: CANVAS_HEIGHT,
            chunk_w: CHUNK_WIDTH,
            chunk_h: CHUNK_HEIGHT,
        })
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin)
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .add_plugins(ShapePlugin)
        .add_plugins(EasingsPlugin)
        .add_systems(Startup, (setup, setup_debug).chain())
        .add_systems(Update, (change_destination, mouse_input))
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

fn mouse_input(
    mut commands: Commands,
    simul: ResMut<Simulation>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut agent_query: Query<(Entity, &Agent, &Chunkable, &Transform)>,
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let (camera, camera_transform) = q_camera.single();

    let window = q_window.single();
    let mut coords: Vec2 = Vec2::new(CANVAS_HEIGHT, CANVAS_WIDTH);

    if let Some(world_position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
        .map(|ray| ray.origin.truncate())
    {
        coords = world_position;
    }

    let clamped_coords = coords.clamp(
        Vec2::new(0. - simul.canvas_w / 2., 0. - simul.canvas_w / 2.),
        Vec2::new(simul.canvas_w / 2., simul.canvas_w / 2.),
    );
    let chunk = simul.get_chunk_coords_vec(clamped_coords);
    for entity in simul.get_chunk_entities(chunk) {
        let (entity, _agent, _chunkable, transform) = agent_query.get_mut(entity).unwrap();

        let mut easing_component = transform.ease_to(
            Transform::from_translation(Vec3::new(clamped_coords.x, clamped_coords.y, 10.)),
            EASING,
            EasingType::Once {
                duration: TRAVEL_DURATION,
            },
        );

        easing_component.state = EasingState::Play;

        commands.entity(entity).insert(easing_component);
    }
}

fn setup_debug(
    mut commands: Commands,
    mut simul: ResMut<Simulation>,
    asset_server: Res<AssetServer>,
) {
    let limits = simul.get_chunk_limits();
    for y in 0..limits.y {
        for x in 0..limits.x {
            let coords = simul.get_global_coords(ChunkCoordinates::new(x, y));
            spawn_debug_square(&mut commands, coords, simul.chunk_w, simul.chunk_h);
        }
    }
    spawn_agents(&mut commands, &asset_server, &mut simul);
}

fn change_destination(
    mut commands: Commands,
    simul: ResMut<Simulation>,
    mut removed: RemovedComponents<EasingComponent<Transform>>,
    mut agent_query: Query<(&mut Agent, &Chunkable, &Transform)>,
) {
    for entity in removed.read() {
        let (mut agent, chunkable, transform) = agent_query.get_mut(entity).unwrap();

        let limits = simul.get_chunk_limits();

        let x_dev = ((chunkable.coords.x as f32 + ((random::<f32>() - 0.5) * 2.).round()) as usize)
            .clamp(0, limits.x - 1);
        let y_dev = ((chunkable.coords.y as f32 + ((random::<f32>() - 0.5) * 2.).round()) as usize)
            .clamp(0, limits.y - 1);

        let n_chunk = ChunkCoordinates::new(x_dev, y_dev);

        let coords = simul.get_global_coords(n_chunk);

        let r_x = (random::<f32>() - 0.5) * simul.chunk_w + coords.x;
        let r_y = (random::<f32>() - 0.5) * simul.chunk_h + coords.y;

        // println!("{}, {}", x_dev, y_dev);
        let destination = Vec3::new(r_x, r_y, 10.);
        agent.destination = destination;

        let mut easing_component = transform.ease_to(
            Transform::from_translation(destination),
            EASING,
            EasingType::Once {
                duration: TRAVEL_DURATION,
            },
        );

        easing_component.state = EasingState::Play;

        commands.entity(entity).insert(easing_component);
    }
}

fn spawn_agents(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    simul: &mut ResMut<Simulation>,
) {
    let limits = simul.get_chunk_limits();
    let chunk_entity_count = AGENT_COUNT / limits.x / limits.y;
    for y in 0..limits.y {
        for x in 0..limits.x {
            let chunk_coords = ChunkCoordinates::new(x, y);
            let coords = simul.get_global_coords(ChunkCoordinates::new(x, y));
            for _ in 0..chunk_entity_count {
                let r_x = (random::<f32>() - 0.5) * simul.chunk_w + coords.x;
                let r_y = (random::<f32>() - 0.5) * simul.chunk_h + coords.y;
                let destination = Vec3::new(r_x, r_y, 10.);
                let entity = commands
                    .spawn((
                        ChunkableBundle {
                            chunkable: Chunkable {
                                coords: chunk_coords,
                            },
                        },
                        SpriteBundle {
                            texture: asset_server.load("sprite.png"),
                            ..default()
                        },
                        Transform {
                            translation: Vec3 {
                                x: r_x,
                                y: r_y,
                                z: 10.,
                            },
                            ..default()
                        }
                        .ease_to(
                            Transform::from_translation(Vec3::new(
                                destination.x,
                                destination.y,
                                10.,
                            )),
                            EASING,
                            EasingType::Once {
                                duration: TRAVEL_DURATION,
                            },
                        ),
                        Agent { destination },
                    ))
                    .id();
                simul.add_entity(chunk_coords, entity);
            }
        }
    }
}

fn spawn_debug_square(commands: &mut Commands, coords: Vec2, chunk_w: f32, chunk_h: f32) {
    let shape = shapes::Rectangle {
        extents: Vec2::new(chunk_w, chunk_h),
        origin: RectangleOrigin::CustomCenter(coords),
    };
    commands.spawn((
        ShapeBundle {
            path: GeometryBuilder::build_as(&shape),
            ..default()
        },
        Stroke::new(Color::RED, 1.0),
    ));
}
