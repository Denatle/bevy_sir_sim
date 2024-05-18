//! Renders a 2D scene containing a single, moving sprite.

use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ShapePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, sprite_movement)
        .run();
}

#[derive(Component)]
enum DirectionHorizontal {
    Right,
    Left,
}

#[derive(Component)]
enum DirectionVertical {
    Up,
    Down,
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("icon.png"),
            transform: Transform::from_xyz(-100., 0., 10.),
            ..default()
        },
        DirectionVertical::Up,
        DirectionHorizontal::Right
    ));
    commands.spawn((
        SpriteBundle {
            texture: asset_server.load("icon.png"),
            transform: Transform::from_xyz(100., 0., 10.),
            ..default()
        },
        DirectionVertical::Down,
        DirectionHorizontal::Left
    ));
}

/// The sprite is animated by changing its translation depending on the time that has passed since
/// the last frame.
fn sprite_movement(
    mut commands: Commands,
    time: Res<Time>,
    lines: Query<Entity, With<Fill>>,
    mut sprites: Query<(
        &mut DirectionVertical,
        &mut DirectionHorizontal,
        &mut Transform)>,
    window: Query<&Window>) {
    let window = window.single();
    let speed = 500. * time.delta_seconds();

    for entity in &lines {
        commands.entity(entity).despawn()
    };

    for (mut d_vert, mut d_horn, mut transform) in &mut sprites {
        let shape = shapes::Line(Vec2::new(transform.translation.x, transform.translation.y), Vec2::new(0., 0.));
        commands.spawn((
            ShapeBundle {
                path: GeometryBuilder::build_as(&shape),
                ..default()
            },
            Fill::color(Color::CYAN),
            Stroke::new(Color::WHITE, 10.),
        ));
        match *d_vert {
            DirectionVertical::Up => transform.translation.y += speed,
            DirectionVertical::Down => transform.translation.y -= speed,
        }
        match *d_horn {
            DirectionHorizontal::Right => transform.translation.x += speed,
            DirectionHorizontal::Left => transform.translation.x -= speed
        }

        if transform.translation.y > window.height() / 2. - 128. {
            *d_vert = DirectionVertical::Down;
        } else if transform.translation.y < -window.height() / 2. + 128. {
            *d_vert = DirectionVertical::Up;
        }

        if transform.translation.x > window.width() / 2. - 128. {
            *d_horn = DirectionHorizontal::Left;
        } else if transform.translation.x < -window.width() / 2. + 128. {
            *d_horn = DirectionHorizontal::Right;
        }
    }
}
