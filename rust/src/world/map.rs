use crate::{
    GameData,
    world::generator::{OverWorldGenerator, TerrainGenerator},
};

use super::units::*;
use crossbeam_channel::{Receiver, Sender, bounded};
use hecs::World;
use noise::{Fbm, Perlin};
use raylib::prelude::*;
use rayon::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{
    collections::HashMap,
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
#[derive(Debug, Default)]
pub struct BlockMapDrawBuffer {
    pub sprites: HashMap<WorldBlockPos, Vec<(Vector3, Rectangle)>>,
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

impl BlockMapDrawBuffer {
    pub fn register(&mut self, wpos: WorldBlockPos, pos: Vector3, rect: Rectangle) {
        if let Some(list) = self.sprites.get_mut(&wpos) {
            list.push((pos, rect));
        } else {
            self.sprites.insert(wpos, vec![(pos, rect)]);
        }
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

        let width = (4.0 / cam_zoom).clamp(1.0, 4.0) as i32;
        let height = (3.0 / cam_zoom).clamp(1.0, 3.0) as i32;

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
    pub fn update(&mut self, _dt: f32, data: &GameData) {
        // get view space for updating chunks
        let (start, end) = Self::view_space(data.camera.target, data.camera.zoom);
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
    pub fn draw(
        &self,
        d: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        buffer: &mut BlockMapDrawBuffer,
        data: &GameData,
    ) {
        let (start, end) = Self::view_space(data.camera.target, data.camera.zoom);

        // all chunks in view
        for y in start.y..=end.y {
            for x in start.x..=end.x {
                self.draw_chunk(d, buffer, &data.atlas, ChunkPos { x, y });
            }
        }
    }
    /// draw the chunk at `ChunkPos`
    #[inline(always)]
    pub fn draw_chunk(
        &self,
        draw: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
        buffer: &mut BlockMapDrawBuffer,
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
                            if let Some(rects) = buffer.sprites.get(&wpos) {
                                for (pos, rect) in rects {
                                    let dst = Rectangle {
                                        x: pos.x * TILE_SIZE as f32,
                                        y: (pos.y - pos.z) * TILE_SIZE as f32,
                                        width: TILE_SIZE as f32,
                                        height: TILE_SIZE as f32,
                                    };
                                    draw.draw_texture_pro(
                                        atlas,
                                        rect,
                                        dst,
                                        Vector2::zero(),
                                        0.0,
                                        Color::WHITE,
                                    );
                                }
                            }
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
pub fn update_map(world: &mut World, data: &mut GameData, dt: f32) {
    for block_map in world.query_mut::<&mut BlockMap>() {
        block_map.update(dt, data);
    }
}
pub fn draw_map(
    world: &mut World,
    d: &mut RaylibMode2D<'_, RaylibDrawHandle<'_>>,
    buffer: &mut BlockMapDrawBuffer,
    data: &GameData,
) {
    for block_map in world.query_mut::<&mut BlockMap>() {
        block_map.draw(d, buffer, data);
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
