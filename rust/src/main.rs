#![allow(dead_code)]
pub mod player;
pub mod world;

use player::Player;
use raylib::prelude::*;
use std::sync::Arc;
use world::{
    map::{BlockData, BlockKind, BlockMap, BlockSet},
    units::{TILE_SIZE, WorldBlockPos},
};

use crate::world::{
    generator::OverWorldGenerator,
    map::{AutoBlockFn, at47},
};

pub struct Game {
    rl: RaylibHandle,
    thread: RaylibThread,
    atlas: Texture2D,
    player: Player,
    world: BlockMap,
    camera: Camera2D,
    selected: u8,
}
impl Default for Game {
    fn default() -> Self {
        let (mut rl, thread) = raylib::init().size(640, 480).title("PicoCraft").build();
        let atlas = rl.load_texture(&thread, "assets/tileset.png").unwrap();
        let at47_arc = Arc::new(at47) as AutoBlockFn;
        Self {
            rl,
            thread,
            atlas,
            player: Player::default(),
            world: BlockMap::new(
                BlockSet {
                    data: vec![
                        // GRASS
                        BlockData {
                            atlas_pos: (0, 0).into(),
                            kind: BlockKind::Block(at47_arc.clone()),
                        },
                        // STONE
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
                },
                OverWorldGenerator::default(),
                42,
            ),
            camera: Camera2D {
                zoom: 0.25,
                ..Default::default()
            },
            selected: 1,
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
            z: 1,
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
            self.world.set_block(mwpos, self.selected);
        } else if self
            .rl
            .is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT)
        {
            self.world.set_block(mwpos, 0);
        }
    }
    pub fn edit_move(&mut self, dt: f32) {
        const SPEED: f32 = 200.0;
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
        // self.player.update(dt);
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
