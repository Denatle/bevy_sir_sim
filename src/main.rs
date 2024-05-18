use bevy::prelude::*;
use bevy::sprite::{MaterialMesh2dBundle, Mesh2dHandle};
use bevy_prototype_lyon::prelude::*;
use rand::prelude::*;

use crate::types::{Person, Simulation};

mod types;

const ENTITY_COUNT: usize = 1500;
const CANVAS_WIDTH: f32 = 1100.;
const CANVAS_HEIGHT: f32 = 1100.;
const CHUNK_SIZE: f32 = 100.;

// TODO: !!! SET ENTITIES TO THE VECTOR ON SECTOR CHANGE !!!

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(
            WindowPlugin {
                primary_window: Some(
                    Window {
                        title: "Simulator".into(),
                        resolution: (1100., 1100.).into(),
                        ..default()
                    }
                ),
                ..default()
            }))
        .insert_resource(Simulation {
            canvas_w: CANVAS_WIDTH,
            canvas_h: CANVAS_HEIGHT,
            chunk_size: CHUNK_SIZE,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(1., 1., 1.)))
        .add_plugins(ShapePlugin)
        .add_systems(Startup, (setup_grid, (setup_camera, setup_debug)).chain())
        .add_systems(Update, move_squares)
        .run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup_grid(
    mut commands: Commands,
    mut simul: ResMut<Simulation>,
) {
    let (x_lim, y_lim) = simul.get_chunk_limits();
    for y in 0..y_lim {
        simul.chunks.push(vec![]);
        for x in 0..x_lim {
            let (r_x, r_y) = simul.get_global_coords(x, y);
            spawn_debug_square(&mut commands, r_x, r_y, simul.chunk_size);
            simul.chunks[y].push(vec![])
        }
    }
}

fn setup_debug(
    mut commands: Commands,
    mut simul: ResMut<Simulation>,
    squares: Query<(&mut Person, &mut Transform)>,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<ColorMaterial>>,
) {
    spawn_debug_entity(&mut commands, meshes, materials, &mut simul);
    squares.iter().for_each(|(_person, transform)| {
        let (n_x, n_y) = simul.get_chunk_coords(transform.translation.x, transform.translation.y);
        simul.chunks[n_x][n_y].iter().for_each(|entity| {
            println!("{:?}: {n_x}, {n_y}", entity)
        });
    });
}

fn move_squares(
    // simul: ResMut<Simulation>,
    time: Res<Time>,
    mut squares: Query<(&mut Person, &mut Transform)>,
) {
    squares.iter_mut().for_each(|(_person, mut transform)| {
        let mut rng = thread_rng();
        let x: f32 = rng.gen();
        let y: f32 = rng.gen();
        // println!("{}", x);
        transform.translation.x += (x - 0.5) * time.delta().as_millis() as f32;
        transform.translation.y += (y - 0.5) * time.delta().as_millis() as f32;

        transform.translation.x = transform.translation.x.clamp(0. - CANVAS_WIDTH / 2., CANVAS_WIDTH / 2.);
        transform.translation.y = transform.translation.y.clamp(0. - CANVAS_HEIGHT / 2., CANVAS_HEIGHT / 2.);

        // let (n_x, n_y) = simul.get_chunk_coords(transform.translation.x, transform.translation.y);
        // println!("{n_x}, {n_y}");
    });
}

fn spawn_debug_entity(commands: &mut Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<ColorMaterial>>, simul: &mut ResMut<Simulation>) {
    for i in 0..ENTITY_COUNT {
        let color = Color::hsl(360. * i as f32 / ENTITY_COUNT as f32, 0.95, 0.5);
        let entity = commands.spawn((
            MaterialMesh2dBundle {
                mesh: Mesh2dHandle(meshes.add(Rectangle::new(7., 7.))),
                material: materials.add(color),
                transform: Transform::from_xyz(
                    0.,
                    0.,
                    0.,
                ),
                ..default()
            },
            Person
        )).id();
        let (n_x, n_y) = simul.get_chunk_coords(-300., -300.);
        simul.add_entity(n_x, n_y, entity);
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
