use bevy::prelude::{Component, Vec3};

#[derive(Component)]
pub struct Agent {
    pub type_: AgentType,
    pub speed: f32,
    pub destination: Vec3,
    pub is_travelling: bool,
}

#[derive(Component)]
pub struct CursorAgent;

#[derive(PartialEq)]
pub enum AgentType {
    Main,
    NearMain,
    FarMain,
}