use std::env;

use bevy::prelude::*;
use bevy::window::WindowMode;
use bevy_pancam::{PanCam, PanCamPlugin};
use bevy_prototype_lyon::prelude::*;
use iyes_perf_ui::prelude::*;
use rand::prelude::*;

use crate::types::{Agent, AgentType, ChunkCoordinates, CursorAgent, Simulation};

mod types;

const CANVAS_WIDTH: f32 = 1500.;
const CANVAS_HEIGHT: f32 = 1500.;
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
        .add_systems(Update, (move_cursor, change_destination, move_agents, (chunk_agents, color_agents).chain()))
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default()).insert(
        PanCam {
            grab_buttons: vec![MouseButton::Middle], // which buttons should drag the camera
            enabled: true, // when false, controls are disabled. See toggle example.
            zoom_to_cursor: true, // whether to zoom towards the mouse or the center of the screen
            min_scale: 0.3, // prevent the camera from zooming too far in
            max_scale: Some(4.5), // prevent the camera from zooming too far out
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
    asset_server: Res<AssetServer>,
) {
    spawn_agents(&mut commands, &asset_server, &mut simul);
    spawn_cursor(&mut commands, &asset_server, &mut simul);
}

fn move_cursor(
    mut simul: ResMut<Simulation>,
    mut squares: Query<(Entity, &mut Agent, &mut Transform)>,
    q_window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform)>,
) {
    for (entity, mut agent, mut transform) in squares.iter_mut() {
        if agent.type_ != AgentType::Main {
            continue;
        }
        let (camera, camera_transform) = q_camera.single();

        let window = q_window.single();
        let mut coords: Vec2 = Vec2::new(-500., -500.);

        if let Some(world_position) = window.cursor_position()
            .and_then(|cursor| camera.viewport_to_world(camera_transform, cursor))
            .map(|ray| ray.origin.truncate())
        {
            coords = world_position;
        }

        transform.translation.x = coords.x.clamp(0. - simul.canvas_w / 2., simul.canvas_w / 2.);
        transform.translation.y = coords.y.clamp(0. - simul.canvas_h / 2., simul.canvas_h / 2.);

        let coords = simul.get_chunk_coords(transform.translation.x, transform.translation.y);
        if coords != agent.coords {
            simul.change_entity_sector(entity, agent.coords, coords);
        }
        agent.coords = coords;
    };
}

fn move_agents(
    mut simul: ResMut<Simulation>,
    time: Res<Time>,
    mut squares: Query<(Entity, &mut Agent, &mut Transform)>,
) {
    for (entity, mut agent, mut transform) in squares.iter_mut() {
        if agent.type_ == AgentType::Main {
            continue;
        }
        let old_translation = transform.translation;
        transform.translation += (agent.destination - old_translation) * agent.speed * time.delta().as_secs_f32();
        let coords = simul.get_chunk_coords(transform.translation.x, transform.translation.y);
        if coords != agent.coords {
            simul.change_entity_sector(entity, agent.coords, coords);
        }
        agent.coords = coords;
    };
}

fn change_destination(
    simul: ResMut<Simulation>,
    mut agent_query: Query<(&mut Agent, &Transform), Without<CursorAgent>>,
) {
    for (mut agent, transform) in agent_query.iter_mut() {
        if agent.type_ != AgentType::FarMain {
            continue;
        }
        agent.type_ = AgentType::FarMain;
        if agent.destination.round() == transform.translation.round() {
            let x: f32 = (random::<f32>() - 0.5) * (simul.canvas_w / 2.);
            let y: f32 = (random::<f32>() - 0.5) * (simul.canvas_h / 2.);
            let destination = Vec3::new(x, y, 0.);
            agent.destination = destination
        }
    }
}

fn chunk_agents(
    simul: ResMut<Simulation>,
    mut agent_query: Query<&mut Agent, Without<CursorAgent>>,
    cursor_agent: Query<(Entity, &Agent, &Transform), With<CursorAgent>>,
) {
    let (cursor_entity, cursor, cursor_transform) = cursor_agent.single();

    for mut agent in agent_query.iter_mut() {
        if agent.coords != cursor.coords {
            agent.type_ = AgentType::FarMain;
        }
    }

    let chunk_entities = simul.get_chunk_entities(cursor.coords);
    for entity in chunk_entities {
        if cursor_entity == entity {
            continue;
        }
        let mut agent = agent_query
            .get_mut(entity).unwrap();
        agent.type_ = AgentType::NearMain;
        agent.destination = cursor_transform.translation
    }
}

fn color_agents(mut squares: Query<(&mut Sprite, &mut Agent)>) {
    for (mut sprite, agent) in squares.iter_mut() {
        match agent.type_ {
            AgentType::Main => { sprite.color = Color::rgb(1., 1., 0.); }
            AgentType::FarMain => { sprite.color = Color::rgb(0., 0., 1.); }
            AgentType::NearMain => { sprite.color = Color::rgb(1., 0., 1.); }
        }
    }
}

fn spawn_cursor(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    simul: &mut ResMut<Simulation>,
) {
    let coords = simul.get_chunk_coords(0., 0.);
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
            type_: AgentType::Main,
            speed: 0.,
            is_travelling: false,
            destination: Vec3::new(0., 0., 0.),
        },
        CursorAgent
    )).id();
    simul.add_entity(coords, entity);
}

fn spawn_agents(commands: &mut Commands, asset_server: &Res<AssetServer>, simul: &mut ResMut<Simulation>) {
    let coords = simul.get_chunk_coords(0., 0.);
    for _ in 0..simul.entity_count {
        let x: f32 = (random::<f32>() - 0.5) * (simul.canvas_w / 2.);
        let y: f32 = (random::<f32>() - 0.5) * (simul.canvas_h / 2.);
        let destination = Vec3::new(x, y, 0.);
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
                type_: AgentType::FarMain,
                speed: 3.,
                is_travelling: true,
                destination,
            },
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
