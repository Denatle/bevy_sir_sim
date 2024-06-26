use bevy::prelude::*;

pub struct Chunking {
    pub entity_count: usize,
    pub canvas_w: f32,
    pub canvas_h: f32,
    pub chunk_w: f32,
    pub chunk_h: f32,
}

impl Plugin for Chunking {
    fn build(&self, app: &mut App) {
        app.insert_resource(Simulation {
            entity_count: self.entity_count,
            canvas_w: self.canvas_w,
            canvas_h: self.canvas_h,
            chunk_w: self.chunk_w,
            chunk_h: self.chunk_h,
            ..default()
        })
        .add_systems(Startup, setup_grid)
        .add_systems(Update, update_chunkables);
    }
}

fn setup_grid(mut simul: ResMut<Simulation>) {
    let limits = simul.get_chunk_limits();
    for y in 0..limits.y {
        simul.chunks.push(vec![]);
        for _x in 0..limits.x {
            simul.chunks[y].push(vec![])
        }
    }
}

fn update_chunkables(
    mut simul: ResMut<Simulation>,
    mut squares: Query<(Entity, &mut Chunkable, &Transform)>,
) {
    for (entity, mut chunkable, transform) in squares.iter_mut() {
        let coords = simul.get_chunk_coords(transform.translation.x, transform.translation.y);
        if coords != chunkable.coords {
            simul.change_entity_sector(entity, chunkable.coords, coords);
        }
        chunkable.coords = coords;
    }
}

#[derive(Resource, Default)]
pub struct Simulation {
    pub entity_count: usize,
    pub canvas_w: f32,
    pub canvas_h: f32,
    pub chunk_w: f32,
    pub chunk_h: f32,
    pub chunks: Vec<Vec<Vec<Entity>>>,
}

impl Simulation {
    pub fn get_chunk_coords(&self, x: f32, y: f32) -> ChunkCoordinates {
        let x = x + self.canvas_w / 2.;
        let y = y + self.canvas_h / 2.;
        let r_x = (x / self.canvas_w * (self.canvas_w / self.chunk_w)).floor();
        let r_y = (y / self.canvas_h * (self.canvas_h / self.chunk_h)).floor();
        ChunkCoordinates {
            x: (r_x as usize).clamp(0, self.get_chunk_limits().x - 1),
            y: (r_y as usize).clamp(0, self.get_chunk_limits().y - 1),
        }
    }

    pub fn get_chunk_limits(&self) -> ChunkCoordinates {
        ChunkCoordinates {
            x: self.canvas_w as usize / self.chunk_w as usize,
            y: self.canvas_h as usize / self.chunk_h as usize,
        }
    }

    pub fn get_global_coords(&self, coords: ChunkCoordinates) -> (f32, f32) {
        let x = coords.x as f32 * self.chunk_w - self.canvas_w / 2. + self.chunk_w / 2.;
        let y = coords.y as f32 * self.chunk_h - self.canvas_h / 2. + self.chunk_h / 2.;
        (x, y)
    }

    pub fn change_entity_sector(
        &mut self,
        entity: Entity,
        old: ChunkCoordinates,
        new: ChunkCoordinates,
    ) {
        self.chunks[old.y][old.x]
            .iter()
            .position(|x| *x == entity)
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
pub struct Chunkable {
    pub coords: ChunkCoordinates,
}

#[derive(Bundle)]
pub struct ChunkableBundle {
    pub chunkable: Chunkable,
}

#[derive(PartialOrd, PartialEq, Copy, Clone, Debug)]
pub struct ChunkCoordinates {
    pub x: usize,
    pub y: usize,
}

impl ChunkCoordinates {
    pub fn new(x: usize, y: usize) -> Self {
        ChunkCoordinates { x, y }
    }
}
