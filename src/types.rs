use bevy::prelude::{Component, Entity, Resource};

#[derive(Resource, Default)]
pub struct Simulation {
    pub canvas_w: f32,
    pub canvas_h: f32,
    pub chunk_size: f32,
    pub chunks: Vec<Vec<Vec<Entity>>>,
}

impl Simulation {
    pub fn get_chunk_coords(&self, x: f32, y: f32) -> (usize, usize) {
        let x = x + self.canvas_w / 2.;
        let y = y + self.canvas_h / 2.;
        let r_x = (x / self.canvas_w * (self.canvas_w / self.chunk_size)).floor();
        let r_y = (y / self.canvas_h * (self.canvas_h / self.chunk_size)).floor();
        // println!("chunk_coord:\n{x}, {y}: {r_x}, {r_y}");
        (r_x as usize, r_y as usize)
    }
    pub fn get_chunk_limits(&self) -> (usize, usize) {
        (self.canvas_w as usize / self.chunk_size as usize,
         self.canvas_h as usize / self.chunk_size as usize)
    }
    pub fn get_global_coords(&self, x: usize, y: usize) -> (f32, f32) {
        let r_x = x as f32 * self.chunk_size - self.canvas_w / 2. + self.chunk_size / 2.;
        let r_y = y as f32 * self.chunk_size - self.canvas_h / 2. + self.chunk_size / 2.;
        (r_x, r_y)
    }
    pub fn change_sector_entity(&self) { todo!() }
    pub fn add_entity(&mut self, x: usize, y: usize, entity: Entity) { 
        self.chunks[y][x].push(entity) 
    }
}

#[derive(Component)]
pub struct Person;
