pub mod player;

use crate::{
    GameData, GameEvent,
    raylib::prelude::*,
    world::{
        map::BlockMapDrawBuffer,
        units::{AtlasPos, TILE_SIZE, WorldBlockPos},
    },
};
use hecs::{Entity, World};
use std::{collections::HashMap, fmt::Debug, sync::Arc};

#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub pos: Vector3,
    pub size: Vector3,
}
#[derive(Debug, Clone, PartialEq)]
/// Events for Bodies
pub enum BodyEvent {
    Overlap { a: Entity, b: Entity },
}
impl From<BodyEvent> for GameEvent {
    fn from(val: BodyEvent) -> Self {
        GameEvent::BodyEvent(val)
    }
}
impl BodyEvent {
    pub fn update(&self, world: &mut World, _dt: f32) {
        match self {
            BodyEvent::Overlap { a, b } => {
                // we already know they are overlapping
                let Ok(mut a) = world.get::<&mut Body>(*a) else {
                    return;
                };
                let Ok(b) = world.get::<&Body>(*b) else {
                    return;
                };
                // horizontal resolve
                if a.pos.x < b.pos.x {
                    a.pos.x = b.pos.x - a.size.x;
                } else if a.pos.x > b.pos.x {
                    a.pos.x = b.pos.x + b.size.x;
                }
                // vertical resolve
                if a.pos.y < b.pos.y {
                    a.pos.y = b.pos.y - a.size.y;
                } else if a.pos.y > b.pos.y {
                    a.pos.y = b.pos.y + b.size.y;
                }
                // height resolve
                if a.pos.z < b.pos.z {
                    a.pos.z = b.pos.z - a.size.z;
                } else if a.pos.z > b.pos.z {
                    a.pos.z = b.pos.z + b.size.z;
                }
            }
        }
    }
}
impl Body {
    #[inline(always)]
    pub fn overlap(&self, other: &Self) -> bool {
        self.pos.x < other.pos.x + other.size.x
            && self.pos.x + self.size.x > other.pos.x
            && self.pos.y < other.pos.y + other.size.y
            && self.pos.y + self.size.y > other.pos.y
            && self.pos.z < other.pos.z + other.size.z
            && self.pos.z + self.size.z > other.pos.z
    }
    #[inline(always)]
    pub fn update_overlap(world: &mut World, data: &mut GameData) {
        let mut snap: Vec<(Entity, Body)> = Vec::new();
        // collect all the bodies
        for (ent, body) in world.query::<(Entity, &Body)>().iter() {
            snap.push((ent, body.clone()));
        }
        if snap.len() <= 1 {
            // nothing that can overlap
            return;
        }
        // find overlaps
        for (ent_a, body_a) in snap.iter() {
            for (ent_b, body_b) in snap.iter() {
                if *ent_a == *ent_b {
                    continue;
                }
                if body_a.overlap(body_b) {
                    data.push_event(BodyEvent::Overlap {
                        a: *ent_a,
                        b: *ent_b,
                    });
                }
            }
        }
    }
}
// PHYSICS
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Physics {
    pub vel: Vector3,
}
pub const GRAVITY: f32 = 10.0;
impl Physics {
    #[inline(always)]
    pub fn update_gravity(world: &mut World, dt: f32) {
        for (body, physics) in world.query_mut::<(&mut Body, &mut Physics)>() {
            physics.vel.z -= GRAVITY * dt;
            body.pos += physics.vel;
            // naiv collision for now
            if body.pos.z < 2.0 {
                body.pos.z = 2.0;
                physics.vel.z = 0.0;
            }
        }
    }
    #[inline(always)]
    pub fn update(world: &mut World, dt: f32) {
        Self::update_gravity(world, dt);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AtlasSprite {
    pub atlas_pos: AtlasPos,
}
impl AtlasSprite {
    #[inline(always)]
    pub fn source(&self) -> Rectangle {
        Rectangle {
            x: self.atlas_pos.x as f32 * TILE_SIZE as f32,
            y: self.atlas_pos.y as f32 * TILE_SIZE as f32,
            width: TILE_SIZE as f32,
            height: TILE_SIZE as f32,
        }
    }
    #[inline(always)]
    pub fn draw_with_body(world: &mut World, d: &mut BlockMapDrawBuffer, _data: &GameData) {
        for (body, atlas_sprite) in world.query::<(&Body, &Self)>().iter() {
            let src = atlas_sprite.source();
            d.register(
                WorldBlockPos {
                    x: body.pos.x as i32,
                    y: body.pos.y as i32,
                    z: body.pos.z as i32,
                },
                body.pos,
                src,
            );
            // d.draw_texture_pro(&data.atlas, src, dst, Vector2::zero(), 0.0, Color::WHITE);
        }
    }
}
pub type AnimationsType = HashMap<&'static str, (&'static [AtlasPos], f32)>;
#[derive(Debug, Default, Clone)]
pub struct AtlasSpriteAnimation {
    pub animations: Arc<AnimationsType>,
    pub current: &'static str,
    pub time: f32,
    pub flip_h: bool,
    pub flip_v: bool,
}
impl AtlasSpriteAnimation {
    pub fn switch_animation(&mut self, new: &'static str) {
        if new != self.current {
            self.time = 0.0;
            self.current = new;
        }
    }
    #[inline(always)]
    pub fn source(&self) -> Rectangle {
        let (arr, scale) = *self.animations.get(&self.current).unwrap();
        let len = arr.len();
        let atlas_pos = arr[(self.time * scale).floor() as usize % len];
        Rectangle {
            x: atlas_pos.x as f32 * TILE_SIZE as f32,
            y: atlas_pos.y as f32 * TILE_SIZE as f32,
            width: if self.flip_h {
                -(TILE_SIZE as f32)
            } else {
                TILE_SIZE as f32
            },
            height: if self.flip_v {
                -(TILE_SIZE as f32)
            } else {
                TILE_SIZE as f32
            },
        }
    }
    #[inline(always)]
    pub fn update_animation(world: &mut World, dt: f32) {
        for atlas_sprite_animation in world.query_mut::<&mut AtlasSpriteAnimation>() {
            atlas_sprite_animation.time += dt;
        }
    }
    #[inline(always)]
    pub fn draw_with_body(world: &mut World, d: &mut BlockMapDrawBuffer, _data: &GameData) {
        for (body, atlas_sprite_animation) in world.query::<(&Body, &AtlasSpriteAnimation)>().iter()
        {
            let src = atlas_sprite_animation.source();
            d.register(
                WorldBlockPos {
                    x: body.pos.x as i32,
                    y: body.pos.y as i32,
                    z: body.pos.z as i32,
                },
                body.pos,
                src,
            );
        }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Inventory {}
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Controller {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub speed: f32,
}
impl Controller {
    #[inline(always)]
    pub fn any(&self) -> bool {
        self.left || self.right || self.up || self.down
    }
    #[inline(always)]
    pub fn update(rl: &mut RaylibHandle, world: &mut World, dt: f32) {
        Self::update_input(rl, world);
        Self::update_move(world, dt);
    }
    #[inline(always)]
    pub fn update_input(rl: &mut RaylibHandle, world: &mut World) {
        for controller in world.query_mut::<&mut Controller>() {
            controller.left = rl.is_key_down(KeyboardKey::KEY_A);
            controller.right = rl.is_key_down(KeyboardKey::KEY_D);
            controller.up = rl.is_key_down(KeyboardKey::KEY_W);
            controller.down = rl.is_key_down(KeyboardKey::KEY_S);
        }
    }
    #[inline(always)]
    pub fn update_move(world: &mut World, dt: f32) {
        for (physics, controller) in world.query_mut::<(&mut Physics, &mut Controller)>() {
            let acc = Vector3 {
                x: if controller.right { 1. } else { 0. } - if controller.left { 1. } else { 0. },
                y: if controller.down { 1. } else { 0. } - if controller.up { 1. } else { 0. },
                ..Default::default()
            };
            physics.vel = acc.normalized() * controller.speed * dt;
        }
    }
}

#[inline(always)]
pub fn update_all(rl: &mut RaylibHandle, world: &mut World, data: &mut GameData, dt: f32) {
    Controller::update(rl, world, dt);
    Body::update_overlap(world, data);
    Physics::update(world, dt);
    AtlasSpriteAnimation::update_animation(world, dt);
    player::Player::update(world, data);
}
#[inline(always)]
pub fn draw_all(world: &mut World, d: &mut BlockMapDrawBuffer, data: &GameData) {
    AtlasSprite::draw_with_body(world, d, data);
    AtlasSpriteAnimation::draw_with_body(world, d, data);
}
