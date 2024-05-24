use bevy::prelude::*;
use bevy_prototype_lyon::draw::Stroke;
use bevy_prototype_lyon::entity::ShapeBundle;
use bevy_prototype_lyon::geometry::GeometryBuilder;
use bevy_prototype_lyon::prelude::RectangleOrigin;
use bevy_prototype_lyon::shapes;

pub struct Chunking {
    pub entity_count: usize,
    pub canvas_w: f32,
    pub canvas_h: f32,
    pub chunk_size: f32,
}

impl Plugin for Chunking {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(Simulation {
                entity_count: self.entity_count,
                canvas_w: self.canvas_w,
                canvas_h: self.canvas_h,
                chunk_size: self.chunk_size,
                ..default()
            })
            .add_systems(Startup, setup_grid)
            .add_systems(Update, update_chunkables);
    }
}


fn setup_grid(
    mut simul: ResMut<Simulation>,
) {
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
    };
}


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
        let r_x = (x / self.canvas_w * (self.canvas_w / self.chunk_size)).floor();
        let r_y = (y / self.canvas_h * (self.canvas_h / self.chunk_size)).floor();
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
        ChunkCoordinates {
            x,
            y,
        }
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
