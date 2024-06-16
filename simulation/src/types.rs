
use bevy::prelude::{Component, Vec3};

#[derive(Component)]
pub struct Agent {
    pub destination: Vec3,
}