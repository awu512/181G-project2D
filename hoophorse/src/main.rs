use std::rc::Rc;
use winit;

use engine;
use engine::animations::AnimationSet;
use engine::sprite::{Action, Character, Sprite};
use engine::tiles::*;
use engine::types::*;

const PLAYER_WIDTH: i32 = 20;
const PLAYER_HEIGHT: i32 = 32;
pub const WIDTH: usize = 320;
pub const HEIGHT: usize = 320;
const TILE_SZ: i32 = 16;

struct Assets {
    spritesheet: Rc<Image>,
    tilesheet: Rc<Image>,
    tileset: Rc<Tileset>,
    tilemap: Tilemap,
    splash: Rc<Image>,
}

struct State {
    player: Rect,
    sprite: Sprite,
    animation_set: AnimationSet,
    flip: bool,
    jumping: bool,
    speedup_factor: usize,
    vx: f32,
    vy: f32,
    ax: f32,
    ay: f32,
    ball: Rect,
    ball_shot: bool,
    bpx: f32,
    bpy: f32,
    bvx: f32,
    bvy: f32,
    bay: f32,
    meter: Rect,
    metering: bool,
    basket: Rect,
    splash_counter: u8,
}

struct Game {}

fn main() {
    engine::eng::go::<Game>();
}

impl engine::eng::Game for Game {
    type Assets = Assets;
    type State = State;
    fn new() -> (State, Assets) {
        let tilesheet = Rc::new(Image::from_file(std::path::Path::new(
            "content/tilesheet.png",
        )));
        let spritesheet = Rc::new(Image::from_file(std::path::Path::new(
            "content/spritesheet.png",
        )));
        let tileset = Rc::new(Tileset::new(
            vec![
                Tile { solid: true },
                Tile { solid: true },
                Tile { solid: true },
                Tile { solid: true },
                Tile { solid: true },
                Tile { solid: false },
                Tile { solid: false },
                Tile { solid: false },
                Tile { solid: false },
                Tile { solid: false },
            ],
            tilesheet.clone(),
        ));
        let map = Tilemap::new(
            Vec2i { x: 0, y: 0 },
            (20, 20),
            tileset.clone(),
            vec![
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 0, 0, 0, 0, 0, 0, 0, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0, 0,
                6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 6, 6, 6, 6, 6, 6, 6, 6,
                6, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 9, 6, 6, 6, 6, 6, 6, 6, 6, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 8, 5, 5, 5, 5, 5, 5, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 2, 2, 2,
                2, 2, 2, 3, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4, 4,
                4, 4, 4, 4, 4, 4, 4, 4,
            ],
        );

        let splash = Rc::new(Image::from_file(std::path::Path::new("content/splash.png")));
        let assets = Assets {
            spritesheet,
            tilesheet,
            tileset,
            tilemap: map,
            splash,
        };
        let state = State::new(Character::Mario);
        (state, assets)
    }

    fn update(state: &mut State, _assets: &mut Assets, now_keys: &[bool], prev_keys: &[bool]) {
        use winit::event::VirtualKeyCode;

        // Keyboard Event
        if now_keys[VirtualKeyCode::Up as usize]
            && !prev_keys[VirtualKeyCode::Up as usize]
            && !state.jumping
        {
            state.vy = -5.0;
            state.jumping = true;
            // Only update the animation if it was previously something else.
            if state.sprite.action != Action::Jump {
                state
                    .sprite
                    .set_animation(state.animation_set.play_animation(Action::Jump));
            }
        }

        if now_keys[VirtualKeyCode::Down as usize] {}
        if now_keys[VirtualKeyCode::Left as usize] {
            if state.vx > -2.0 {
                state.ax = -0.2;
            } else {
                state.ax = 0.0
            }
            state.flip = true;
            // Only update the animation if it was previously something else.
            if state.sprite.action != Action::Walk {
                state
                    .sprite
                    .set_animation(state.animation_set.play_animation(Action::Walk));
            }
        } else if now_keys[VirtualKeyCode::Right as usize] {
            if state.vx < 2.0 {
                state.ax = 0.2;
            } else {
                state.ax = 0.0
            }
            state.flip = false;
            // Only update the animation if it was previously something else.
            if state.sprite.action != Action::Walk {
                state
                    .sprite
                    .set_animation(state.animation_set.play_animation(Action::Walk));
            }
        } else {
            if state.vx > 0.1 {
                state.ax = -0.1
            } else if state.vx < -0.1 {
                state.ax = 0.1
            } else {
                state.ax = 0.0
            }
        }

        if now_keys[VirtualKeyCode::Space as usize] && !state.ball_shot {
            state.meter.pos.x = state.player.pos.x + state.player.sz.x;
            state.meter.pos.y = state.player.pos.y + state.player.sz.y / 2 - state.meter.sz.y;

            state.metering = true;

            if state.meter.sz.y < 64 {
                state.meter.sz.y += 1;
            }
        }

        if !now_keys[VirtualKeyCode::Space as usize]
            && prev_keys[VirtualKeyCode::Space as usize]
            && !state.ball_shot
        {
            state.ball.pos = state.player.pos;
            state.bpx = state.player.pos.x as f32;
            state.bpy = state.player.pos.y as f32;
            state.bvx = -6.0 * (state.meter.sz.y as f32 / 64.0);
            state.bvy = -4.0;
            state.metering = false;
            state.meter.sz.y = 0;
            state.ball_shot = true;
        }

        if state.vx as i32 == 0 && state.vy as i32 == 0 {
            state
                .sprite
                .set_animation(state.animation_set.play_animation(Action::Stand));
        }
    }

    fn render(state: &mut State, assets: &mut Assets, fb2d: &mut Image) {
        assets.tilemap.draw(fb2d);
        fb2d.bitblt(
            &assets.spritesheet,
            state.sprite.play_animation(&state.speedup_factor),
            state.player.pos,
            state.flip,
        );

        state.vx += state.ax;
        state.vy += state.ay;
        state.player.move_by(state.vx as i32, state.vy as i32);

        let mut ovs = vec![];
        for i in 0..3 {
            for j in 0..3 {
                let p = Vec2i {
                    x: state.player.pos.x + i * (state.player.sz.x / 2),
                    y: state.player.pos.y + j * (state.player.sz.y / 2),
                };
                let r = assets.tilemap.tile_at(p);
                if r.1.solid {
                    let mut ov = Vec2i { x: 0, y: 0 };
                    if state.vx > 0.0 {
                        ov.x = r.0.x - (state.player.pos.x + PLAYER_WIDTH);
                    } else {
                        ov.x = (r.0.x + TILE_SZ) - state.player.pos.x;
                    }

                    if state.vy > 0.0 {
                        ov.y = r.0.y - (state.player.pos.y + PLAYER_HEIGHT);
                    } else {
                        ov.y = (r.0.y + TILE_SZ) - state.player.pos.y;
                    }

                    ovs.push(ov);
                }
            }
        }

        let mut disps = Vec2i { x: 0, y: 0 };
        let mut resolved = false;
        for ov in ovs.iter() {
            // Touching but not overlapping
            if ov.x == 0 && ov.y == 0 {
                resolved = true;
                // Maybe track "I'm touching it on this side or that side"
                break;
            }
            // Is this more of a horizontal collision... (and we are allowed to displace horizontally)
            if ov.x.abs() <= ov.y.abs() && ov.x.signum() != -disps.x.signum() {
                // Record that we moved by o.x, to avoid contradictory moves later
                disps.x += ov.x;
                // Actually move player pos
                state.player.pos.x += ov.x;
                state.vx = 0.0;
                // Mark collision for the player as resolved.
                resolved = true;
                break;
                // or is it more of a vertical collision (and we are allowed to displace vertically)
            } else if ov.y.abs() <= ov.x.abs() && ov.y.signum() != -disps.y.signum() {
                disps.y += ov.y;
                state.player.pos.y += ov.y;
                state.vy = 0.0;
                state.jumping = false;
                resolved = true;
                break;
            } else {
                // otherwise, we can't actually handle this displacement because we had a contradictory
                // displacement earlier in the frame.
            }
        }
        // Couldn't resolve collision, player must be squashed or trapped (e.g. by a moving platform)
        if !resolved {
            // In your game, this might mean killing the player character or moving them somewhere else
        }

        // check to make sure player is in screen bounds
        if state.player.pos.x < 0 {
            state.player.pos.x = 0
        }
        if state.player.pos.x > WIDTH as i32 - state.player.sz.x {
            state.player.pos.x = WIDTH as i32 - state.player.sz.x
        }
        if state.player.pos.y < 0 {
            state.player.pos.y = 0;
        }
        if state.player.pos.y > HEIGHT as i32 - state.player.sz.y {
            state.player.pos.y = HEIGHT as i32 - state.player.sz.y;
        }

        // BALL CODE
        if state.ball_shot {
            if state.basket.contains_point({
                Vec2i {
                    x: state.bpx as i32 + state.ball.sz.x / 2,
                    y: state.bpy as i32 + state.ball.sz.y / 2,
                }
            }) {
                state.ball_shot = false;
                state.splash_counter = 30;
            }

            if (state.bpx + state.bpy) as i32 > 0
                && (state.bpy as i32) < (HEIGHT as i32)
                && state.ball_shot
            {
                state.bvy += state.bay;

                state.bpx += state.bvx;
                state.bpy += state.bvy;

                state.ball.pos.x = state.bpx as i32;
                state.ball.pos.y = state.bpy as i32;

                fb2d.draw_ball(&state.ball);
            } else {
                state.ball_shot = false;
                state.bvx = 0.0;
                state.bvy = 0.0;
            }
        }

        if state.metering {
            fb2d.draw_rect(&state.meter, (255, 255, 255, 255));
        }

        if state.splash_counter > 0 {
            fb2d.bitblt(
                &assets.splash,
                Rect {
                    pos: Vec2i { x: 16, y: 0 },
                    sz: Vec2i { x: 16, y: 16 },
                },
                Vec2i { x: 0, y: 256 },
                false,
            );
            state.splash_counter -= 1;
        }
    }
}

impl State {
    pub fn new(character: Character) -> Self {
        let animation_set = AnimationSet::new(character);
        let sprite = Sprite {
            character: character,
            action: Action::Stand,
            animation_state: animation_set.play_animation(Action::Stand),
            shape: Rect {
                pos: Vec2i { x: 20, y: 20 },
                sz: Vec2i {
                    x: PLAYER_WIDTH as i32,
                    y: PLAYER_HEIGHT as i32,
                },
            },
        };
        let speedup_factor = 5; // this acts more like a slow down factor.
        let player = Rect {
            pos: Vec2i {
                x: (WIDTH as i32) / 4 - PLAYER_WIDTH / 2,
                y: (HEIGHT as i32) - 48 - PLAYER_HEIGHT,
            },
            sz: Vec2i {
                x: PLAYER_WIDTH,
                y: PLAYER_HEIGHT,
            },
        };

        // BALL
        let ball = Rect {
            pos: Vec2i { x: 0, y: 0 },
            sz: Vec2i { x: 8, y: 8 },
        };

        // POWER METER
        let meter = Rect {
            pos: Vec2i { x: 0, y: 0 },
            sz: Vec2i { x: 4, y: 0 },
        };

        let basket = Rect {
            pos: Vec2i { x: 4, y: 260 },
            sz: Vec2i { x: 8, y: 8 },
        };

        State {
            player: player,
            sprite: sprite,
            animation_set: animation_set,
            flip: false,
            jumping: false,
            speedup_factor: speedup_factor,
            vx: 0.0,
            vy: 0.0,
            ax: 0.0,
            ay: 0.2,
            ball,
            ball_shot: false,
            bpx: 0.0,
            bpy: 0.0,
            bvx: 0.0,
            bvy: 0.0,
            bay: 0.2,
            meter,
            metering: false,
            basket,
            splash_counter: 0,
        }
    }
}
