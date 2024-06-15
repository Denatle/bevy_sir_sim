use bevy::prelude::{Component, Vec3};

#[derive(Component)]
pub struct Agent {
    pub speed: f32,
    pub destination: Vec3,
}