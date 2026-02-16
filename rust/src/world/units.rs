use raylib::prelude::*;

/// static tile size
pub const TILE_SIZE: usize = 32;
/// static chunk size
pub const CHUNK_SIZE: usize = 16;
/// static chunk size
pub const CHUNK_HEIGHT: usize = 8;
/// static chunk volume for flat lists
pub const CHUNK_VOLUME: usize = CHUNK_SIZE * CHUNK_SIZE * CHUNK_HEIGHT;

/// represents a tile position in the atlas based on `TILE_SIZE`
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtlasPos {
    pub x: u8,
    pub y: u8,
}
impl AtlasPos {
    /// turns into a source rectangle for the atlas image
    #[inline(always)]
    pub fn source(self) -> Rectangle {
        Rectangle {
            x: self.x as f32 * TILE_SIZE as f32,
            y: self.y as f32 * TILE_SIZE as f32,
            width: TILE_SIZE as f32,
            height: TILE_SIZE as f32,
        }
    }
}
impl From<(u8, u8)> for AtlasPos {
    #[inline(always)]
    fn from((x, y): (u8, u8)) -> Self {
        Self { x, y }
    }
}
/// represents a block position in a chunk
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkBlockPos {
    pub x: usize,
    pub y: usize,
    pub z: usize,
}
impl ChunkBlockPos {
    /// returns the flat index of the position
    #[inline(always)]
    pub fn idx(self) -> usize {
        self.z * (CHUNK_SIZE * CHUNK_SIZE) + self.y * CHUNK_SIZE + self.x
    }
    /// converts it to world position based on the chunk position `cpos`
    #[inline(always)]
    pub fn to_world(self, cpos: ChunkPos) -> WorldBlockPos {
        WorldBlockPos {
            x: cpos.x * CHUNK_SIZE as i32 + self.x as i32,
            y: cpos.y * CHUNK_SIZE as i32 + self.y as i32,
            z: self.z as i32,
        }
    }
}
/// represents the position of a chunk
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
}
/// represents the position of a tile in the world
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WorldBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}
impl From<WorldBlockPos> for ChunkBlockPos {
    #[inline(always)]
    fn from(w: WorldBlockPos) -> Self {
        ChunkBlockPos {
            x: w.x.rem_euclid(CHUNK_SIZE as i32) as usize,
            y: w.y.rem_euclid(CHUNK_SIZE as i32) as usize,
            z: w.z.rem_euclid(CHUNK_SIZE as i32) as usize,
        }
    }
}
impl From<WorldBlockPos> for ChunkPos {
    #[inline(always)]
    fn from(w: WorldBlockPos) -> Self {
        ChunkPos {
            x: w.x.div_euclid(CHUNK_SIZE as i32),
            y: w.y.div_euclid(CHUNK_SIZE as i32),
        }
    }
}
