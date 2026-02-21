use hecs::World;
use raylib::math::Vector3;

use crate::{SCREEN_HEIGHT, SCREEN_WIDTH, components::*};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Player {
    state: PlayerState,
    dir: Vector2,
}
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum PlayerState {
    #[default]
    Idle,
    Walk,
    Jump,
}

impl Player {
    pub fn update_camera(world: &mut World, data: &mut GameData) {
        for body in world.query::<&Body>().with::<&Player>().iter() {
            data.camera.target = Vector2 {
                x: body.pos.x * TILE_SIZE as f32 - (SCREEN_WIDTH as f32 / data.camera.zoom) / 2.0
                    + TILE_SIZE as f32 / 2.0,
                y: (body.pos.y - body.pos.z) * TILE_SIZE as f32
                    - (SCREEN_HEIGHT as f32 / data.camera.zoom) / 2.0
                    + TILE_SIZE as f32 / 2.0,
            }
        }
    }
    pub fn update_animation(world: &mut World) {
        for (player, controller, anim) in
            world.query_mut::<(&mut Player, &Controller, &mut AtlasSpriteAnimation)>()
        {
            if controller.any() {
                player.dir = Vector2 {
                    x: if controller.right { 1. } else { 0. }
                        - if controller.left { 1. } else { 0. },
                    y: if controller.down { 1. } else { 0. } - if controller.up { 1. } else { 0. },
                };
                player.state = PlayerState::Walk;
            } else {
                player.state = PlayerState::Idle;
            }
            match player.state {
                PlayerState::Idle => {
                    if player.dir.y > 0.0 {
                        anim.switch_animation("idle-down");
                    } else if player.dir.y < 0.0 {
                        anim.switch_animation("idle-up");
                    } else if player.dir.x > 0.0 {
                        anim.switch_animation("idle-side");
                        anim.flip_h = false;
                    } else if player.dir.x < 0.0 {
                        anim.switch_animation("idle-side");
                        anim.flip_h = true;
                    }
                }
                PlayerState::Walk => {
                    if player.dir.y > 0.0 {
                        anim.switch_animation("walk-down");
                    } else if player.dir.y < 0.0 {
                        anim.switch_animation("walk-up");
                    } else if player.dir.x > 0.0 {
                        anim.switch_animation("walk-side");
                        anim.flip_h = false;
                    } else if player.dir.x < 0.0 {
                        anim.switch_animation("walk-side");
                        anim.flip_h = true;
                    }
                }
                PlayerState::Jump => todo!(),
            }
        }
    }
    pub fn update(world: &mut World, data: &mut GameData) {
        Self::update_animation(world);
        Self::update_camera(world, data);
    }
}
#[inline(always)]
pub fn spawn_player(world: &mut World) {
    let mut map: AnimationsType = HashMap::default();
    const WALK_SPEED: f32 = 10.0;
    map.insert("idle-side", (&[AtlasPos { x: 9, y: 0 }], 1.0));
    map.insert(
        "walk-side",
        (
            &[
                AtlasPos { x: 9, y: 1 },
                AtlasPos { x: 9, y: 0 },
                AtlasPos { x: 9, y: 2 },
                AtlasPos { x: 9, y: 0 },
            ],
            WALK_SPEED,
        ),
    );
    map.insert("idle-up", (&[AtlasPos { x: 9, y: 3 }], 1.0));
    map.insert(
        "walk-up",
        (
            &[
                AtlasPos { x: 9, y: 4 },
                AtlasPos { x: 9, y: 3 },
                AtlasPos { x: 9, y: 5 },
                AtlasPos { x: 9, y: 3 },
            ],
            WALK_SPEED,
        ),
    );
    map.insert("idle-down", (&[AtlasPos { x: 9, y: 6 }], 1.0));
    map.insert(
        "walk-down",
        (
            &[
                AtlasPos { x: 9, y: 7 },
                AtlasPos { x: 9, y: 6 },
                AtlasPos { x: 9, y: 8 },
                AtlasPos { x: 9, y: 6 },
            ],
            WALK_SPEED,
        ),
    );
    world.spawn((
        Body {
            pos: Vector3::zero(),
            size: Vector3::one(),
        },
        Physics::default(),
        AtlasSpriteAnimation {
            animations: Arc::new(map),
            current: "idle-side",
            flip_h: true,
            ..Default::default()
        },
        Controller {
            speed: 5.0,
            ..Default::default()
        },
        Player::default(),
    ));
}
