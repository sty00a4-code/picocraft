#![allow(dead_code)]
extern crate crossbeam_channel;
extern crate hecs;
extern crate noise;
extern crate rand;
extern crate raylib;
extern crate rayon;
extern crate rustc_hash;

pub mod components;
pub mod world;

use std::collections::VecDeque;

use crate::world::{generator::OverWorldGenerator, map};
use hecs::World;
use raylib::prelude::*;
use world::map::{BlockMap, BlockSet};

pub const SCREEN_WIDTH: i32 = (1920.0 / 1.5) as i32;
pub const SCREEN_HEIGHT: i32 = (1080.0 / 1.5) as i32;

#[derive(Debug, Clone, PartialEq)]
pub enum GameEvents {
    BodyEvent(components::BodyEvents),
}
pub struct GameData {
    events: VecDeque<GameEvents>,
    camera: Camera2D,
    atlas: Texture2D,
    selected: u8,
}
pub struct Game {
    rl: RaylibHandle,
    thread: RaylibThread,
    world: World,
    data: GameData,
}
impl Default for Game {
    fn default() -> Self {
        let (mut rl, thread) = raylib::init()
            .size(SCREEN_WIDTH, SCREEN_HEIGHT)
            .title("PicoCraft")
            .build();
        let atlas = rl.load_texture(&thread, "assets/tileset.png").unwrap();
        let mut world = World::new();
        world.spawn((BlockMap::new(
            BlockSet::normal(),
            OverWorldGenerator::default(),
            42,
        ),));
        components::player::spawn_player(&mut world);
        Self {
            rl,
            thread,
            world,
            data: GameData {
                events: VecDeque::default(),
                camera: Camera2D {
                    zoom: 2.0,
                    ..Default::default()
                },
                atlas,
                selected: 1,
            },
        }
    }
}
impl Game {
    pub fn edit(&mut self, dt: f32) {
        self.edit_move(dt);
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
        self.data.camera.target += acc.normalized() * SPEED / self.data.camera.zoom * dt;
    }
    pub fn update(&mut self, dt: f32) {
        self.edit(dt);
        map::update_map(&mut self.world, &mut self.data, dt);
        components::update_all(&mut self.rl, &mut self.world, &mut self.data, dt);
    }
    pub fn draw(&mut self) {
        let fps = self.rl.get_fps();
        let mut d = self.rl.begin_drawing(&self.thread);
        d.clear_background(Color::SKYBLUE);
        {
            let mut draw: RaylibMode2D<'_, RaylibDrawHandle<'_>> = d.begin_mode2D(self.data.camera);
            map::draw_map(&mut self.world, &mut draw, &self.data);
            components::draw_all(&mut self.world, &mut draw, &self.data);
        }
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
