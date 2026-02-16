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
        for z in 0..CHUNK_SIZE {
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
}
impl Default for OverWorldGenerator {
    fn default() -> Self {
        Self {
            block_height: NoiseLayers {
                layers: [NoiseConfig {
                    freq: 0.01,
                    amp: 1.0,
                    gain: 1.0,
                    offset: [100.0, 200.0],
                }],
                scales: [1.0],
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
        _chunk_data: &Self::ChunkData,
    ) -> Block {
        let height = self
            .block_height
            .get([x as f64 * 0.01, y as f64 * 0.01], perlin);
        if (height > 0.2 && z == 2) || (height > 0.05 && z == 1) {
            return 1;
        } else if height > 0.0 && z == 1 {
            return 3;
        }
        0
    }
}
