use bevy::prelude::{Component, Entity, Resource, Vec3};

#[derive(Resource, Default)]
pub struct Simulation {
    pub entity_count: usize,
    pub canvas_w: f32,
    pub canvas_h: f32,
    pub chunk_size: f32,
    pub chunks: Vec<Vec<Vec<Entity>>>,
}

impl Simulation {
    pub fn get_chunk_coords(&self, x: f32, y: f32) -> ChunkCoordinates {
        let x = x + self.canvas_w / 2.;
        let y = y + self.canvas_h / 2.;
        let r_x = (x / self.canvas_w * (self.canvas_w / self.chunk_size)).floor()
            ;
        let r_y = (y / self.canvas_h * (self.canvas_h / self.chunk_size)).floor();
        // println!("chunk_coord:\n{x}, {y}: {r_x}, {r_y}");
        ChunkCoordinates {
            x: (r_x as usize).clamp(0, self.get_chunk_limits().x - 1),
            y: (r_y as usize).clamp(0, self.get_chunk_limits().y - 1),
        }
    }
    pub fn get_chunk_limits(&self) -> ChunkCoordinates {
        ChunkCoordinates {
            x: self.canvas_w as usize / self.chunk_size as usize,
            y: self.canvas_h as usize / self.chunk_size as usize,
        }
    }
    pub fn get_global_coords(&self, coords: ChunkCoordinates) -> (f32, f32) {
        let x = coords.x as f32 * self.chunk_size - self.canvas_w / 2. + self.chunk_size / 2.;
        let y = coords.y as f32 * self.chunk_size - self.canvas_h / 2. + self.chunk_size / 2.;
        (x, y)
    }
    pub fn change_entity_sector(
        &mut self,
        entity: Entity,
        old: ChunkCoordinates,
        new: ChunkCoordinates,
    ) {
        self.chunks[old.y][old.x].iter()
            .position(|x| { *x == entity })
            .map(|t| self.chunks[old.y][old.x].remove(t));
        self.chunks[new.y][new.x].push(entity);
    }
    pub fn add_entity(&mut self, coords: ChunkCoordinates, entity: Entity) {
        self.chunks[coords.y][coords.x].push(entity)
    }
    pub fn get_chunk_entities(&self, coords: ChunkCoordinates) -> Vec<Entity> {
        self.chunks[coords.y][coords.x].clone()
    }
}

#[derive(Component)]
pub struct Agent {
    pub type_: AgentType,
    pub speed: f32,
    pub destination: Vec3,
    pub is_travelling: bool,
    pub coords: ChunkCoordinates,
}

#[derive(Component)]
pub struct CursorAgent;

#[derive(PartialEq)]
pub enum AgentType {
    Main,
    NearMain,
    FarMain,
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct ChunkCoordinates {
    pub x: usize,
    pub y: usize,
}

impl ChunkCoordinates {
    pub(crate) fn new(x: usize, y: usize) -> Self {
        ChunkCoordinates {
            x,
            y,
        }
    }
}
    