/// src/world/generator.rs
use super::{
    map::{Block, Chunk},
    units::*,
};
use noise::{Fbm, NoiseFn, Perlin};

pub trait TerrainGenerator {
    type ChunkData;
    fn get_chunk_data(&self, cpos: ChunkPos, perlin: &Fbm<Perlin>) -> Self::ChunkData;
    fn gen_chunk(&self, cpos: ChunkPos, perlin: &Fbm<Perlin>) -> Chunk {
        let mut chunk = Chunk::new_empty();
        let chunk_data = self.get_chunk_data(cpos, perlin);
        for z in 0..CHUNK_HEIGHT {
            for y in 0..CHUNK_SIZE {
                for x in 0..CHUNK_SIZE {
                    let ctpos = ChunkBlockPos { x, y, z };
                    let wpos = ctpos.to_world(cpos);
                    chunk.set(ctpos, self.gen_block(wpos, perlin, &chunk_data));
                }
            }
        }
        chunk
    }
    fn gen_block(
        &self,
        wpos: WorldBlockPos,
        perlin: &Fbm<Perlin>,
        chunk_data: &Self::ChunkData,
    ) -> Block;
}

pub trait GetNoise<const DIM: usize> {
    fn get(&self, pos: [f64; DIM], perlin: &Fbm<Perlin>) -> f64;
}
#[derive(Debug, Clone, PartialEq)]
pub struct NoiseConfig<const DIM: usize> {
    pub freq: f64,
    pub amp: f64,
    pub gain: f64,
    pub offset: [f64; DIM],
}
impl GetNoise<2> for NoiseConfig<2> {
    #[inline(always)]
    fn get(&self, pos: [f64; 2], perlin: &Fbm<Perlin>) -> f64 {
        let p = [0, 1].map(|i| pos[i] + self.offset[i] * self.freq);
        perlin.get(p) * self.amp * self.gain
    }
}
impl GetNoise<3> for NoiseConfig<3> {
    #[inline(always)]
    fn get(&self, pos: [f64; 3], perlin: &Fbm<Perlin>) -> f64 {
        let p = [0, 1, 2].map(|i| pos[i] + self.offset[i] * self.freq);
        perlin.get(p) * self.amp * self.gain
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct NoiseLayers<const SIZE: usize, const DIM: usize> {
    pub layers: [NoiseConfig<DIM>; SIZE],
    pub scales: [f64; SIZE],
}
impl<const SIZE: usize> GetNoise<2> for NoiseLayers<SIZE, 2> {
    fn get(&self, pos: [f64; 2], perlin: &Fbm<Perlin>) -> f64 {
        self.layers
            .iter()
            .zip(self.scales)
            .map(|(layer, scale)| layer.get(pos, perlin) * scale)
            .sum::<f64>()
    }
}
impl<const SIZE: usize> GetNoise<3> for NoiseLayers<SIZE, 3> {
    fn get(&self, pos: [f64; 3], perlin: &Fbm<Perlin>) -> f64 {
        self.layers
            .iter()
            .zip(self.scales)
            .map(|(layer, scale)| layer.get(pos, perlin) * scale)
            .sum::<f64>()
    }
}

#[derive(Debug)]
pub struct OverWorldGenerator {
    pub block_height: NoiseLayers<1, 2>,
    pub plants: NoiseConfig<2>,
}
impl Default for OverWorldGenerator {
    fn default() -> Self {
        Self {
            block_height: NoiseLayers {
                layers: [NoiseConfig {
                    freq: 0.01,
                    amp: 1.,
                    gain: 1.,
                    offset: [100., 200.],
                }],
                scales: [1.],
            },
            plants: NoiseConfig {
                freq: 1.0,
                amp: 1.0,
                gain: 1.,
                offset: [0., 0.],
            },
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum OverWorldBiom {
    Plains,
}
impl TerrainGenerator for OverWorldGenerator {
    type ChunkData = OverWorldBiom;
    fn get_chunk_data(&self, _cpos: ChunkPos, _perlin: &Fbm<Perlin>) -> Self::ChunkData {
        OverWorldBiom::Plains
    }
    fn gen_block(
        &self,
        WorldBlockPos { x, y, z }: WorldBlockPos,
        perlin: &Fbm<Perlin>,
        chunk_data: &Self::ChunkData,
    ) -> Block {
        // generator.rs -> gen_block(...)
        let height = self
            .block_height
            .get([x as f64 * 0.01, y as f64 * 0.01], perlin);
        let plants = self.plants.get([x as f64, y as f64], perlin);

        match chunk_data {
            OverWorldBiom::Plains => {
                if z == 4 {
                    if height > 0.6 {
                        return Block::Rock;
                    }
                    return Block::default();
                }
                if z == 3 {
                    if height > 0.4 {
                        return Block::Rock;
                    } else if height > 0.2 {
                        if plants > 0.2 {
                            return Block::Tree;
                        } else if plants > 0.15 {
                            return Block::BerryBush;
                        } else if plants > 0.1 {
                            return Block::Bush;
                        }
                    }
                    return Block::default();
                }
                if z == 2 {
                    if height > 0.2 {
                        return Block::Grass;
                    } else if height > 0.05 {
                        if plants > 0.2 {
                            return Block::Tree;
                        } else if plants > 0.15 {
                            return Block::BerryBush;
                        } else if plants > 0.1 {
                            return Block::Bush;
                        }
                    }
                    return Block::default();
                }
                // surface layer handling
                if z == 1 {
                    if height > 0.05 {
                        return Block::Grass;
                    } else if height > 0.0 {
                        return Block::Sand;
                    } else {
                        return Block::default();
                    }
                }

                // lower layers
                if z < 1 {
                    if height > 0.05 {
                        return Block::Grass;
                    }
                    if height > 0.0 {
                        return Block::Sand;
                    }
                }

                Block::default()
            }
        }
    }
}
/// END src/world/generator.rs

/// src/world/mod.rs
pub mod generator;
pub mod map;
pub mod units;
/// END src/world/mod.rs

/// src/world/units.rs
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
/// END src/world/units.rs

/// src/world/map.rs
use crate::world::generator::{OverWorldGenerator, TerrainGenerator};

use super::units::*;
use crossbeam_channel::{Receiver, Sender, bounded};
use noise::{Fbm, Perlin};
use raylib::prelude::*;
use rayon::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{
    fmt::Debug,
    sync::Arc,
    thread::{self, JoinHandle},
};

// ... add these types for thread communication
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ChunkRequest(ChunkPos);

#[derive(Debug)]
pub struct ChunkResponse {
    pub pos: ChunkPos,
    pub chunk: Chunk,
}

/// holds static `BlockData` for every `Block`
#[derive(Debug, Clone)]
pub struct BlockSet {
    pub data: Vec<BlockData>,
}
/// holds the atlas position and the kind of block
#[derive(Debug, Clone)]
pub struct BlockData {
    pub atlas_pos: AtlasPos,
    pub kind: BlockKind,
}
/// defines the kind of block
#[derive(Clone)]
pub enum BlockKind {
    Block(AutoBlockFn),
    Prop,
}
/// universal autotiling function shared pointer
pub type AutoBlockFn = Arc<dyn Fn(Neighbors) -> AtlasPos + Send + Sync>;
/// holds the chunks and a `BlockSet`
pub struct BlockMap {
    pub seed: u32,
    pub blockset: BlockSet,
    pub chunks: FxHashMap<ChunkPos, Chunk>,
    pub generators: Vec<JoinHandle<()>>,
    pub request_tx: Sender<ChunkRequest>,
    pub response_rx: Receiver<ChunkResponse>,
    pub last_view: (ChunkPos, ChunkPos),
}

/// holds blocks and their cashed `BlockNeighbors`
#[derive(Debug)]
pub struct Chunk {
    blocks: [Block; CHUNK_VOLUME],
    neighbors: [BlockNeighbors; CHUNK_VOLUME],
}
/// represents a block type
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Block {
    #[default]
    Air,
    Grass,
    Rock,
    Sand,
    Tree,
    Bush,
    BerryBush,
    Flower,
    Mushroom,
    Stone,
    Iron,
    Diamond,
    Ruby,
}
impl From<Block> for u8 {
    fn from(val: Block) -> Self {
        match val {
            Block::Air => 0,
            Block::Grass => 1,
            Block::Rock => 2,
            Block::Sand => 3,
            Block::Tree => 4,
            Block::Bush => 5,
            Block::BerryBush => 6,
            Block::Flower => 7,
            Block::Mushroom => 8,
            Block::Stone => 9,
            Block::Iron => 10,
            Block::Diamond => 11,
            Block::Ruby => 12,
        }
    }
}
impl TryInto<Block> for u8 {
    type Error = ();
    fn try_into(self) -> Result<Block, Self::Error> {
        match self {
            0 => Ok(Block::Air),
            1 => Ok(Block::Grass),
            2 => Ok(Block::Rock),
            3 => Ok(Block::Sand),
            4 => Ok(Block::Tree),
            5 => Ok(Block::Bush),
            6 => Ok(Block::BerryBush),
            7 => Ok(Block::Flower),
            8 => Ok(Block::Mushroom),
            9 => Ok(Block::Stone),
            10 => Ok(Block::Iron),
            11 => Ok(Block::Diamond),
            12 => Ok(Block::Ruby),
            _ => Err(()),
        }
    }
}
impl BlockSet {
    pub fn normal() -> Self {
        let at47_arc = Arc::new(at47) as AutoBlockFn;
        BlockSet {
            data: vec![
                // GRASS
                BlockData {
                    atlas_pos: (0, 0).into(),
                    kind: BlockKind::Block(at47_arc.clone()),
                },
                // ROCK
                BlockData {
                    atlas_pos: (0, 12).into(),
                    kind: BlockKind::Block(at47_arc.clone()),
                },
                // SAND
                BlockData {
                    atlas_pos: (0, 24).into(),
                    kind: BlockKind::Block(at47_arc.clone()),
                },
                // TREE
                BlockData {
                    atlas_pos: (8, 2).into(),
                    kind: BlockKind::Prop,
                },
                // BUSH
                BlockData {
                    atlas_pos: (7, 3).into(),
                    kind: BlockKind::Prop,
                },
                // BERRY BUSH
                BlockData {
                    atlas_pos: (8, 3).into(),
                    kind: BlockKind::Prop,
                },
                // FLOWER
                BlockData {
                    atlas_pos: (7, 4).into(),
                    kind: BlockKind::Prop,
                },
                // MUSHROOM
                BlockData {
                    atlas_pos: (8, 4).into(),
                    kind: BlockKind::Prop,
                },
                // STONE
                BlockData {
                    atlas_pos: (8, 14).into(),
                    kind: BlockKind::Prop,
                },
                // IRON
                BlockData {
                    atlas_pos: (7, 15).into(),
                    kind: BlockKind::Prop,
                },
                // DIAMOND
                BlockData {
                    atlas_pos: (8, 15).into(),
                    kind: BlockKind::Prop,
                },
                // RUBY
                BlockData {
                    atlas_pos: (7, 16).into(),
                    kind: BlockKind::Prop,
                },
            ],
        }
    }
}
/// represents the 8 surrounding neighbors
pub type Neighbors = u8;
/// represents the 8 surrounding neighbors for the floor and wall
pub type BlockNeighbors = (Neighbors, Neighbors);

impl Chunk {
    #[inline(always)]
    pub fn new_empty() -> Self {
        Self {
            blocks: [Block::default(); CHUNK_VOLUME],
            neighbors: [(0, 0); CHUNK_VOLUME],
        }
    }

    // ... (keep all existing Chunk methods unchanged)

    /// returns `Block` at `ChunkBlockPos`
    #[inline(always)]
    pub fn get(&self, pos: ChunkBlockPos) -> Option<Block> {
        if pos.x < CHUNK_SIZE && pos.y < CHUNK_SIZE && pos.z < CHUNK_SIZE {
            Some(self.blocks.get(pos.idx()).copied().unwrap_or_default())
        } else {
            None
        }
    }

    /// returns `BlockNeighbors` at `ChunkBlockPos`
    #[inline(always)]
    pub fn get_neighbors(&self, pos: ChunkBlockPos) -> Option<BlockNeighbors> {
        if pos.x < CHUNK_SIZE && pos.y < CHUNK_SIZE && pos.z < CHUNK_SIZE {
            Some(self.neighbors.get(pos.idx()).copied().unwrap_or_default())
        } else {
            None
        }
    }

    /// sets `Block` at `ChunkBlockPos`
    #[inline(always)]
    pub fn set(&mut self, pos: ChunkBlockPos, block: Block) {
        if pos.x < CHUNK_SIZE && pos.y < CHUNK_SIZE && pos.z < CHUNK_SIZE {
            let Some(b) = self.blocks.get_mut(pos.idx()) else {
                return;
            };
            *b = block
        }
    }

    /// sets `BlockNeighbors` at `ChunkBlockPos`
    #[inline(always)]
    pub fn set_neighbors(&mut self, pos: ChunkBlockPos, neighbors: BlockNeighbors) {
        if pos.x < CHUNK_SIZE && pos.y < CHUNK_SIZE && pos.z < CHUNK_SIZE {
            let idx = pos.idx();
            unsafe { *self.neighbors.get_unchecked_mut(idx) = neighbors }
        }
    }
}

impl BlockSet {
    /// get the data for any `Block`
    pub fn get_data(&self, gid: Block) -> Option<&BlockData> {
        if gid == Block::default() {
            return None;
        }
        self.data.get((Into::<u8>::into(gid) - 1) as usize)
    }
}

impl BlockMap {
    pub fn new(blockset: BlockSet, generator: OverWorldGenerator, seed: u32) -> Self {
        // threaded chunk generation channels
        let (request_tx, request_rx): (Sender<ChunkRequest>, Receiver<ChunkRequest>) = bounded(64);
        let (response_tx, response_rx): (Sender<ChunkResponse>, Receiver<ChunkResponse>) =
            bounded(64);

        // shared generator
        let generator = Arc::new(generator);

        // spawn generator threads with channels
        let mut generators = Vec::with_capacity(8);
        for _ in 0..8 {
            let request_rx = request_rx.clone();
            let response_tx = response_tx.clone();
            let generator = generator.clone();

            let handle = thread::spawn(move || {
                let perlin = Fbm::<Perlin>::new(seed);
                while let Ok(ChunkRequest(pos)) = request_rx.recv() {
                    let generator = generator.clone();
                    let chunk = generator.gen_chunk(pos, &perlin);
                    let _ = response_tx.send(ChunkResponse { pos, chunk });
                }
            });
            generators.push(handle);
        }

        // drop unused channel references
        drop((request_rx, response_tx));

        Self {
            blockset,
            chunks: FxHashMap::with_capacity_and_hasher(0xffff, FxBuildHasher),
            generators,
            seed,
            request_tx,
            response_rx,
            last_view: (ChunkPos::default(), ChunkPos::default()),
        }
    }

    /// returns `Chunk` at `ChunkPos`
    #[inline(always)]
    pub fn get_chunk(&self, pos: ChunkPos) -> Option<&Chunk> {
        self.chunks.get(&pos)
    }

    /// returns mutable `Chunk` at `ChunkPos`
    #[inline(always)]
    pub fn get_chunk_mut(&mut self, pos: ChunkPos) -> Option<&mut Chunk> {
        self.chunks.get_mut(&pos)
    }

    /// set `Chunk` at `ChunkPos`
    #[inline(always)]
    pub fn set_chunk(&mut self, pos: ChunkPos, chunk: Chunk) {
        self.chunks.insert(pos, chunk);
    }

    /// returns `Block` at `WorldBlockPos`
    #[inline(always)]
    pub fn get_block(&self, pos: WorldBlockPos) -> Option<Block> {
        let cpos: ChunkPos = pos.into();
        let ct: ChunkBlockPos = pos.into();
        self.chunks.get(&cpos).and_then(|chunk| chunk.get(ct))
    }

    /// returns `BlockNeighbors` at `WorldBlockPos`
    #[inline(always)]
    pub fn get_neighbors(&self, pos: WorldBlockPos) -> Option<BlockNeighbors> {
        let cpos: ChunkPos = pos.into();
        let ct: ChunkBlockPos = pos.into();
        self.chunks
            .get(&cpos)
            .and_then(|chunk| chunk.get_neighbors(ct))
    }

    /// sets `Block` at `WorldBlockPos` and updates surrounding chunks
    #[inline(always)]
    pub fn set_block(&mut self, pos: WorldBlockPos, tile: Block) {
        let cpos: ChunkPos = pos.into();
        let ctpos: ChunkBlockPos = pos.into();
        if let Some(chunk) = self.chunks.get_mut(&cpos) {
            chunk.set(ctpos, tile);
        } else {
            let mut chunk = Chunk::new_empty();
            chunk.set(ctpos, tile);
            self.set_chunk(cpos, chunk);
        }

        for dy in -1..=1 {
            for dx in -1..=1 {
                let npos = ChunkPos {
                    x: cpos.x + dx,
                    y: cpos.y + dy,
                };
                if self.get_chunk(npos).is_some() {
                    self.update_chunk_neighbors(npos);
                }
            }
        }
    }

    /// sets `BlockNeighbors` at `WorldBlockPos`
    #[inline(always)]
    pub fn set_neighbors(&mut self, pos: WorldBlockPos, neighbors: BlockNeighbors) {
        let cpos: ChunkPos = pos.into();
        let ctpos: ChunkBlockPos = pos.into();
        if let Some(chunk) = self.chunks.get_mut(&cpos) {
            chunk.set_neighbors(ctpos, neighbors);
        } else {
            let mut chunk = Chunk::new_empty();
            chunk.set_neighbors(ctpos, neighbors);
            self.set_chunk(cpos, chunk);
        }
    }

    /// identifies the viewable chunks based on camera target and zoom
    #[inline(always)]
    pub fn view_space(cam_world_center: Vector2, cam_zoom: f32) -> (ChunkPos, ChunkPos) {
        let denom = TILE_SIZE as f32 * CHUNK_SIZE as f32;
        let chunk_start_x = (cam_world_center.x / denom).floor() as i32;
        let chunk_start_y = (cam_world_center.y / denom).floor() as i32;

        let width = (3.0 / cam_zoom).clamp(1.0, 3.0) as i32;
        let height = (3.0 / cam_zoom).clamp(1.0, 2.0) as i32;

        let chunk_end_x = chunk_start_x + width;
        let chunk_end_y = chunk_start_y + height;
        (
            ChunkPos {
                x: chunk_start_x,
                y: chunk_start_y,
            },
            ChunkPos {
                x: chunk_end_x,
                y: chunk_end_y,
            },
        )
    }

    /// updates every chunk and generates new ones within `view_space`
    pub fn update(&mut self, _dt: f32, camera: &Camera2D) {
        // get view space for updating chunks
        let (start, end) = Self::view_space(camera.target, camera.zoom);
        // receive new chunks from channel
        while let Ok(response) = self.response_rx.try_recv() {
            self.set_chunk(response.pos, response.chunk);
        }
        // only update neighbors if view space changed
        let update_chunk_neighbors = start != self.last_view.0 || end != self.last_view.1;
        // find new chunks to load and update chunk neighbors if necessary
        for y in start.y..=end.y {
            for x in start.x..=end.x {
                let pos = ChunkPos { x, y };
                if self.get_chunk(pos).is_none() {
                    let _ = self.request_tx.send(ChunkRequest(pos));
                } else if update_chunk_neighbors {
                    self.update_chunk_neighbors(pos);
                }
            }
        }
    }

    /// generate a `Chunk` at `ChunkPos` with `seed`
    #[inline(always)]
    pub fn generate_chunk(&self, cpos: ChunkPos) {
        let _ = self.request_tx.send(ChunkRequest(cpos));
    }

    /// update the neighbors of each block in the chunk at `ChunkPos`
    #[inline(always)]
    pub fn update_chunk_neighbors(&mut self, cpos: ChunkPos) {
        // block map for parrallel computing
        let world_ref: &BlockMap = self;

        // compute the neighbors in parrallel
        let new_neighbors: Vec<BlockNeighbors> = (0..CHUNK_VOLUME)
            .into_par_iter()
            .map(|idx| {
                // get chunk block position
                let z = (idx / (CHUNK_SIZE * CHUNK_SIZE)) as i32;
                let y = ((idx / CHUNK_SIZE) % CHUNK_SIZE) as i32;
                let x = (idx % CHUNK_SIZE) as i32;

                let ctpos = ChunkBlockPos {
                    x: x as usize,
                    y: y as usize,
                    z: z as usize,
                };
                let wpos = ctpos.to_world(cpos);

                if let Some(gid) = world_ref.get_block(wpos) {
                    // floor neighbors
                    let mut floor_neighbors: Neighbors = 0;
                    for ny in -1..=1 {
                        for nx in -1..=1 {
                            if nx == 0 && ny == 0 {
                                continue;
                            }
                            let npos = WorldBlockPos {
                                x: wpos.x + nx,
                                y: wpos.y + ny,
                                z: wpos.z,
                            };
                            floor_neighbors = (floor_neighbors << 1)
                                | if world_ref.get_block(npos).unwrap_or_default() == gid {
                                    1
                                } else {
                                    0
                                };
                        }
                    }

                    // wall neighbors with top always present
                    let mut wall_neighbors: Neighbors = 0b111;
                    for dz in 0..=1 {
                        for dx in -1..=1 {
                            if dx == 0 && dz == 0 {
                                continue;
                            }
                            let npos = WorldBlockPos {
                                x: wpos.x + dx,
                                y: wpos.y,
                                z: wpos.z - dz,
                            };
                            wall_neighbors = (wall_neighbors << 1)
                                | if world_ref.get_block(npos).unwrap_or_default() == gid {
                                    1
                                } else {
                                    0
                                };
                        }
                    }
                    wall_neighbors = !(!wall_neighbors | 0b111);
                    (floor_neighbors, wall_neighbors)
                } else {
                    (0, 0)
                }
            })
            .collect();

        if let Some(chunk) = self.get_chunk_mut(cpos) {
            chunk.neighbors.copy_from_slice(&new_neighbors);
        }
    }

    /// draw the world to the screen in `view_space`
    pub fn draw(&self, d: &mut RaylibDrawHandle, atlas: &Texture2D, camera: &Camera2D) {
        let mut draw: RaylibMode2D<'_, RaylibDrawHandle<'_>> = d.begin_mode2D(*camera);

        let (start, end) = Self::view_space(camera.target, camera.zoom);

        // all chunks in view
        for y in start.y..=end.y {
            for x in start.x..=end.x {
                self.draw_chunk(&mut draw, atlas, ChunkPos { x, y });
            }
        }
    }
    /// draw the chunk at `ChunkPos`
    #[inline(always)]
    pub fn draw_chunk(
        &self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        atlas: &Texture2D,
        cpos: ChunkPos,
    ) {
        // chunk is generated
        if let Some(chunk) = self.get_chunk(cpos) {
            // for all blocks
            for z in 0..CHUNK_HEIGHT {
                for y in 0..CHUNK_SIZE {
                    for x in 0..CHUNK_SIZE {
                        let ctpos = ChunkBlockPos { x, y, z };
                        if let Some(gid) = chunk.get(ctpos) {
                            let wpos = ctpos.to_world(cpos);
                            self.draw_tile(&mut *draw, atlas, wpos, gid);
                        }
                    }
                }
            }
        }
    }

    /// draw the tile at `WorldBlockPos`
    #[inline(always)]
    pub fn draw_tile(
        &self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        atlas: &Texture2D,
        wpos: WorldBlockPos,
        gid: Block,
    ) {
        // screen position
        let spos = Vector2 {
            x: wpos.x as f32 * TILE_SIZE as f32,
            y: ((wpos.y - wpos.z) as f32) * TILE_SIZE as f32,
        };

        // above position & block
        let above_pos = WorldBlockPos {
            x: wpos.x,
            y: wpos.y,
            z: wpos.z + 1,
        };
        let above_gid = self.get_block(above_pos).unwrap_or_default();

        if let Some(tile_data) = self.blockset.get_data(gid) {
            // get floor neighbors
            if matches!(tile_data.kind, BlockKind::Block(_)) {
                let (floor_mask, _) = self.get_neighbors(wpos).unwrap_or((0, 0));
                self.draw_tile_floor_cached(draw, atlas, spos, tile_data, floor_mask);
            }
        }

        if let Some(above_data) = self.blockset.get_data(above_gid) {
            // get wall neighbors
            let (_, wall_neighbors) = self.get_neighbors(above_pos).unwrap_or((0, 0));
            self.draw_tile_wall_cached(draw, atlas, spos, above_data, wall_neighbors);
        }
    }

    fn draw_tile_floor_cached(
        &self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        atlas: &Texture2D,
        spos: Vector2,
        BlockData { atlas_pos, kind }: &BlockData,
        floor_neighbors: Neighbors,
    ) {
        let dst = Rectangle::new(spos.x, spos.y, TILE_SIZE as f32, TILE_SIZE as f32);
        let src = match kind {
            BlockKind::Block(autotile) => {
                // autotiling offset
                let offset: AtlasPos = autotile(floor_neighbors);
                // atlas wall position
                AtlasPos {
                    x: atlas_pos.x + offset.x,
                    y: atlas_pos.y + offset.y,
                }
                .source()
            }
            BlockKind::Prop => atlas_pos.source(),
        };
        draw.draw_texture_pro(atlas, src, dst, Vector2::zero(), 0.0, Color::WHITE);
    }

    fn draw_tile_wall_cached(
        &self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        atlas: &Texture2D,
        spos: Vector2,
        BlockData { atlas_pos, kind }: &BlockData,
        wall_neighbors: Neighbors,
    ) {
        let dst = Rectangle::new(spos.x, spos.y, TILE_SIZE as f32, TILE_SIZE as f32);
        let src = match kind {
            BlockKind::Block(autotile) => {
                // autotiling offset
                let offset: AtlasPos = autotile(wall_neighbors);
                // atlas wall position
                AtlasPos {
                    x: atlas_pos.x + offset.x,
                    y: atlas_pos.y + offset.y + 6,
                }
                .source()
            }
            BlockKind::Prop => atlas_pos.source(),
        };
        draw.draw_texture_pro(atlas, src, dst, Vector2::zero(), 0.0, Color::WHITE);
    }
}

impl Debug for BlockKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BlockKind::Block(_) => write!(f, "Block"),
            BlockKind::Prop => write!(f, "Prop"),
        }
    }
}

pub fn at47(neighbors: Neighbors) -> AtlasPos {
    let arr: [bool; 8] = std::array::from_fn(|i| (neighbors & (1 << (7 - i))) != 0);
    const X: bool = true;
    const O: bool = false;
    From::<(u8, u8)>::from(match arr {
        // CLASS 1
        // DOTS
        // X X X
        // X   X
        // X X 0
        [X, X, X, X, X, X, X, O] => (0, 2),
        // X X X
        // X   X
        // 0 X X
        [X, X, X, X, X, O, X, X] => (1, 2),
        // 0 X X
        // X   X
        // X X X
        [O, X, X, X, X, X, X, X] => (1, 3),
        // X X 0
        // X   X
        // X X X
        [X, X, O, X, X, X, X, X] => (0, 3),

        // WIRLS
        // X X X
        // X   X
        // 0 X 0
        [X, X, X, X, X, O, X, O] => (2, 2),
        // 0 X 0
        // X   X
        // X X X
        [O, X, O, X, X, X, X, X] => (3, 2),
        // 0 X X
        // X   X
        // 0 X X
        [O, X, X, X, X, O, X, X] => (3, 3),
        // X X 0
        // X   X
        // X X 0
        [X, X, O, X, X, X, X, O] => (2, 3),

        // CROSS
        // 0 X 0
        // X   X
        // 0 X 0
        [O, X, O, X, X, O, X, O] => (7, 2),

        // DIAGONALS
        // X X 0
        // X   X
        // 0 X X
        [X, X, O, X, X, O, X, X] => (6, 4),
        // 0 X X
        // X   X
        // X X 0
        [O, X, X, X, X, X, X, O] => (6, 5),

        // TRIANGLES
        // 0 X 0
        // X   X
        // 0 X X
        [O, X, O, X, X, O, X, X] => (0, 4),
        // 0 X 0
        // X   X
        // X X 0
        [O, X, O, X, X, X, X, O] => (1, 4),
        // X X 0
        // X   X
        // 0 X 0
        [X, X, O, X, X, O, X, O] => (1, 5),
        // 0 X X
        // X   X
        // 0 X 0
        [O, X, X, X, X, O, X, O] => (0, 5),

        // CLASS 2
        // WIRLS
        // ? 0 ?
        // X   X
        // 0 X 0
        [_, O, _, X, X, O, X, O] => (3, 0),
        // 0 X 0
        // X   X
        // ? 0 ?
        [O, X, O, X, X, _, O, _] => (4, 0),
        // 0 X ?
        // X   0
        // 0 X ?
        [O, X, _, X, O, O, X, _] => (4, 1),
        // ? X 0
        // 0   X
        // ? X 0
        [_, X, O, O, X, _, X, O] => (3, 1),

        // SIDES
        // ? X X
        // 0   X
        // ? X X
        [_, X, X, O, X, _, X, X] => (4, 2),
        // X X ?
        // X   0
        // X X ?
        [X, X, _, X, O, X, X, _] => (5, 2),
        // X X X
        // X   X
        // ? 0 ?
        [X, X, X, X, X, _, O, _] => (5, 3),
        // ? 0 ?
        // X   X
        // X X X
        [_, O, _, X, X, X, X, X] => (4, 3),

        // VERTICAL TURN
        // ? X 0
        // 0   X
        // ? X X
        [_, X, O, O, X, _, X, X] => (2, 4),
        // 0 X ?
        // X   0
        // X X ?
        [O, X, _, X, O, X, X, _] => (3, 4),
        // X X ?
        // X   0
        // 0 X ?
        [X, X, _, X, O, O, X, _] => (3, 5),
        // ? X X
        // 0   X
        // ? X 0
        [_, X, X, O, X, _, X, _] => (2, 5),

        // HORIZONTAL TURN
        // ? 0 ?
        // X   X
        // 0 X X
        [_, O, _, X, X, O, X, X] => (4, 4),
        // ? 0 ?
        // X   X
        // X X 0
        [_, O, _, X, X, X, X, O] => (5, 4),
        // X X 0
        // X   X
        // ? 0 ?
        [X, X, O, X, X, _, O, _] => (5, 5),
        // 0 X X
        // X   X
        // ? 0 ?
        [O, X, X, X, X, _, O, _] => (4, 5),

        // CLASS 3
        // DONUTS
        // ? 0 ?
        // 0   X
        // ? X 0
        [_, O, _, O, X, _, X, O] => (0, 0),
        // ? 0 ?
        // X   0
        // 0 X ?
        [_, O, _, X, O, O, X, _] => (1, 0),
        // 0 X ?
        // X   0
        // ? 0 ?
        [O, X, _, X, O, _, O, _] => (1, 1),
        // ? X 0
        // 0   X
        // ? 0 ?
        [_, X, O, O, X, _, O, _] => (0, 1),

        // CIRCLES
        // ? 0 ?
        // 0   X
        // ? X X
        [_, O, _, O, X, _, X, X] => (7, 0),
        // ? 0 ?
        // X   0
        // X X ?
        [_, O, _, X, O, X, X, _] => (8, 0),
        // X X ?
        // X   0
        // ? 0 ?
        [X, X, _, X, O, _, O, _] => (8, 1),
        // ? X X
        // 0   X
        // ? 0 ?
        [_, X, X, O, X, _, O, _] => (7, 1),

        // CLASS 4
        // LONELY
        // ? 0 ?
        // 0   0
        // ? 0 ?
        [_, O, _, O, O, _, O, _] => (6, 3),

        // BARS
        // ? 0 ?
        // X   X
        // ? 0 ?
        [_, O, _, X, X, _, O, _] => (2, 0),
        // ? X ?
        // 0   0
        // ? X ?
        [_, X, _, O, O, _, X, _] => (2, 1),

        // TIPS
        // ? 0 ?
        // X   0
        // ? 0 ?
        [_, O, _, X, O, _, O, _] => (5, 0),
        // ? X ?
        // 0   0
        // ? 0 ?
        [_, X, _, O, O, _, O, _] => (6, 0),
        // ? 0 ?
        // 0   X
        // ? 0 ?
        [_, O, _, O, X, _, O, _] => (6, 1),
        // ? 0 ?
        // 0   0
        // ? X ?
        [_, O, _, O, O, _, X, _] => (5, 1),

        // FLAT
        _ => (6, 2),
    })
}
/// END src/world/map.rs


/// src/main.rs
#![allow(dead_code)]
pub mod entities;
pub mod world;

use entities::player::Player;
use raylib::prelude::*;
use world::{
    map::{BlockMap, BlockSet},
    units::{TILE_SIZE, WorldBlockPos},
};

use crate::world::{generator::OverWorldGenerator, map::Block};

pub struct Game {
    rl: RaylibHandle,
    thread: RaylibThread,
    atlas: Texture2D,
    world: BlockMap,
    camera: Camera2D,
    selected: u8,
    player: Player,
}
impl Default for Game {
    fn default() -> Self {
        let (mut rl, thread) = raylib::init().size(640, 480).title("PicoCraft").build();
        let atlas = rl.load_texture(&thread, "assets/tileset.png").unwrap();
        Self {
            rl,
            thread,
            atlas,
            world: BlockMap::new(BlockSet::normal(), OverWorldGenerator::default(), 42),
            camera: Camera2D {
                zoom: 0.5,
                ..Default::default()
            },
            selected: 1,
            player: Player::default(),
        }
    }
}
impl Game {
    pub fn edit(&mut self, dt: f32) {
        let m = self.rl.get_mouse_position() / (TILE_SIZE as f32 * self.camera.zoom);
        let (mx, my) = (
            self.camera.target.x as i32 / TILE_SIZE as i32 + m.x as i32,
            self.camera.target.y as i32 / TILE_SIZE as i32 + m.y as i32,
        );
        let mwpos = WorldBlockPos {
            x: mx,
            y: my + 1,
            z: 2,
        };
        self.edit_place(mwpos);
        self.edit_move(dt);
    }
    pub fn edit_place(&mut self, mwpos: WorldBlockPos) {
        let md = self.rl.get_mouse_wheel_move();
        if md > 0.0 {
            self.selected = self.selected.wrapping_add(1).max(1);
        } else if md < 0.0 {
            self.selected = self.selected.wrapping_sub(1).max(1);
        }
        if self.rl.is_mouse_button_down(MouseButton::MOUSE_BUTTON_LEFT) {
            self.world
                .set_block(mwpos, self.selected.try_into().unwrap_or_default());
        } else if self
            .rl
            .is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT)
        {
            self.world.set_block(mwpos, Block::default());
        }
    }
    pub fn edit_move(&mut self, dt: f32) {
        const SPEED: f32 = 300.0;
        let mut acc = Vector2::zero();
        if self.rl.is_key_down(KeyboardKey::KEY_A) {
            acc.x -= 1.0;
        }
        if self.rl.is_key_down(KeyboardKey::KEY_D) {
            acc.x += 1.0;
        }
        if self.rl.is_key_down(KeyboardKey::KEY_W) {
            acc.y -= 1.0;
        }
        if self.rl.is_key_down(KeyboardKey::KEY_S) {
            acc.y += 1.0;
        }
        self.camera.target += acc.normalized() * SPEED / self.camera.zoom * dt;
    }
    pub fn update(&mut self, dt: f32) {
        self.edit(dt);
        self.world.update(dt, &self.camera);
    }
    pub fn draw(&mut self) {
        let fps = self.rl.get_fps();
        let mut d = self.rl.begin_drawing(&self.thread);
        d.clear_background(Color::SKYBLUE);
        self.world.draw(&mut d, &self.atlas, &self.camera);
        // self.player.draw();
        d.draw_text(&fps.to_string(), 5, 5, 32, Color::RED);
    }
    pub fn run(&mut self) {
        while !self.rl.window_should_close() {
            self.update(self.rl.get_frame_time());
            self.draw();
        }
    }
}

fn main() {
    let mut game = Game::default();
    game.run();
}
/// END src/main.rs

/// src/entities/mod.rs
pub mod player;
/// END src/entities/mod.rs

/// src/entities/player.rs
#[derive(Debug, Default)]
pub struct Player {}
/// END src/entities/player.rs


