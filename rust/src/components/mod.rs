pub mod player;

use std::{collections::HashMap, fmt::Debug, hash::Hash, sync::Arc};

use hecs::{Entity, World};

use crate::{
    GameData, GameEvents,
    raylib::prelude::*,
    world::units::{AtlasPos, TILE_SIZE},
};

#[derive(Debug, Clone, PartialEq)]
pub enum BodyEvents {
    Overlap { src: Entity, dst: Entity },
}
impl From<BodyEvents> for GameEvents {
    fn from(val: BodyEvents) -> Self {
        GameEvents::BodyEvent(val)
    }
}
#[derive(Debug, Clone, PartialEq)]
pub struct Body {
    pub pos: Vector3,
    pub size: Vector3,
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
        for (ent, body) in world.query::<(Entity, &Body)>().iter() {
            snap.push((ent, body.clone()));
        }
        if snap.is_empty() {
            return;
        }
        for (ent_a, body_a) in snap.iter() {
            for (ent_b, body_b) in snap.iter() {
                if *ent_a == *ent_b {
                    continue;
                }
                if body_a.overlap(body_b) {
                    data.events.push_back(
                        BodyEvents::Overlap {
                            src: *ent_a,
                            dst: *ent_b,
                        }
                        .into(),
                    );
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
            if body.pos.z < 0.0 {
                body.pos.z = 0.0;
                physics.vel.z = 0.0;
            }
        }
    }
    #[inline(always)]
    pub fn update_collision(world: &mut World) {
        let mut snap: Vec<(Entity, Body)> = Vec::new();
        for (ent, body, _) in world.query::<(Entity, &Body, &Physics)>().iter() {
            snap.push((ent, body.clone()));
        }
        if snap.is_empty() {
            return;
        }
        let mut accum: HashMap<Entity, Vector3> = HashMap::new();
        for (i, (ent_a, body_a)) in snap.iter().enumerate() {
            let mut acc = Vector3::zero();
            let mut len: usize = 0;
            for (j, (_, body_b)) in snap.iter().enumerate() {
                if i == j {
                    continue;
                }
                if !body_a.overlap(body_b) {
                    continue;
                }

                acc += body_a.pos - body_b.pos;
                len += 1;
            }
            if len > 0 {
                let dv = acc / (len as f32);
                accum.entry(*ent_a).and_modify(|v| *v += dv).or_insert(dv);
            }
        }

        for (ent, _, phys) in world.query_mut::<(Entity, &Body, &mut Physics)>() {
            if let Some(dv) = accum.get(&ent).copied() {
                phys.vel += dv;
            }
        }
    }
    #[inline(always)]
    pub fn update(world: &mut World, dt: f32) {
        Self::update_gravity(world, dt);
        Self::update_collision(world);
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
    pub fn draw_with_body(world: &mut World, d: &mut RaylibDrawHandle, data: &GameData) {
        for (body, atlas_sprite) in world.query::<(&Body, &Self)>().iter() {
            let src = atlas_sprite.source();
            let dst = Rectangle {
                x: body.pos.x * TILE_SIZE as f32,
                y: (body.pos.y - body.pos.z) * TILE_SIZE as f32,
                width: TILE_SIZE as f32,
                height: TILE_SIZE as f32,
            };
            d.draw_texture_pro(&data.atlas, src, dst, Vector2::zero(), 0.0, Color::WHITE);
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
    pub fn draw_with_body(world: &mut World, d: &mut RaylibDrawHandle, data: &GameData) {
        for (body, atlas_sprite_animation) in world.query::<(&Body, &AtlasSpriteAnimation)>().iter()
        {
            let src = atlas_sprite_animation.source();
            let dst = Rectangle {
                x: body.pos.x * TILE_SIZE as f32,
                y: (body.pos.y - body.pos.z) * TILE_SIZE as f32,
                width: TILE_SIZE as f32,
                height: TILE_SIZE as f32,
            };
            d.draw_texture_pro(&data.atlas, src, dst, Vector2::zero(), 0.0, Color::WHITE);
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
pub fn draw_all(world: &mut World, d: &mut RaylibDrawHandle, data: &GameData) {
    AtlasSprite::draw_with_body(world, d, data);
    AtlasSpriteAnimation::draw_with_body(world, d, data);
}
