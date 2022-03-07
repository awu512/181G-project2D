use rand::Rng;
use std::rc::Rc;
use winit;

use engine;
use engine::animations::AnimationSet;
use engine::sprite::{Action, Character, Sprite};
use engine::types::*;

const PLAYER_WIDTH: i32 = 32;
const PLAYER_HEIGHT: i32 = 16;
pub const WIDTH: i32 = 320;
pub const HEIGHT: i32 = 320;

const BULLET_VELO: i32 = 1;

const RED: Color = (181, 49, 32, 255);
const BLUE: Color = (74, 206, 222, 255);

const SS_PLAYER: Rect = Rect {
    pos: Vec2i { x: 32, y: 16 },
    sz: Vec2i { x: 32, y: 16 },
};

struct Assets {
    spritesheet: Rc<Image>,
}

struct State {
    player_sprite: Sprite,
    player_bullets: Vec<Rect>,
    vx: f32,
    ax: f32,

    enemies: Vec<Enemy>,
    enemy_bullets: Vec<Rect>,
    evx: i32,

    blockers: Vec<Blocker>,
    shooting_timeout: u8,
  
    game_over: u8
}

impl State {
    pub fn new() -> Self {
        // SPRITES
        let animation_set = AnimationSet::new(Character::SpaceInvader);
        let player_sprite = Sprite {
            character: Character::SpaceInvader,
            action: Action::Glide,
            animation_state: animation_set.play_animation(Action::Glide),
            shape: Rect {
                pos: Vec2i {
                    x: WIDTH / 2 - PLAYER_WIDTH / 2,
                    y: HEIGHT - PLAYER_HEIGHT * 3,
                },
                sz: Vec2i {
                    x: PLAYER_WIDTH,
                    y: PLAYER_HEIGHT,
                },
            },
        };

        let mut enemies = vec![];

        for y in 0..2 {
            for x in 0..8 {
                enemies.push(Enemy::new((x+y) % 2, Vec2i{x,y}));
            }
        }
        let mut blockers = vec![];
        for y in 0..2 {
            for x in 0..12 {
                blockers.push(Blocker::new(Vec2i { x, y }));
            }
        }

        State {
            player_sprite,
            player_bullets: vec![],
            vx: 0.0,
            ax: 0.0,

            enemies,
            enemy_bullets: vec![],
            evx: 1,

            blockers,
            shooting_timeout: 0,

            game_over: 0
        }
    }
}

struct Game {}

struct Enemy {
    style: i32,
    sprite: Sprite,
    rect: Rect,
    alive: bool,
}

impl Enemy {
    pub fn new(style: i32, index: Vec2i) -> Self {
        assert!(index.x < 8, "{} is out of range 8", index.x);
        assert!(index.y < 2, "{} is out of range 2", index.y);
        let character = if style == 1 {
            Character::SpaceInvaderEnemy1
        } else if style == 2 {
            Character::SpaceInvaderEnemy2
        } else {
            Character::SpaceInvaderEnemy1
        };
        let animation_set = AnimationSet::new(character);
        let sprite = Sprite {
            character: character,
            action: Action::Glide,
            animation_state: animation_set.play_animation(Action::Glide),
            shape: Rect {
                pos: Vec2i {
                    x: 64 + 16 * index.x + (16 * index.x / 2),
                    y: 32 + 32 * index.y,
                },
                sz: Vec2i { x: 16, y: 16 },
            },
        };
        Self {
            style,
            sprite: sprite,
            rect: Rect {
                pos: Vec2i {
                    x: 64 + 16 * index.x + (16 * index.x / 2),
                    y: 32 + 32 * index.y,
                },
                sz: Vec2i { x: 16, y: 16 },
            },
            alive: true,
        }
    }

    pub fn shoot(&self) -> Rect {
        Rect {
            pos: Vec2i {
                x: self.rect.pos.x + self.rect.sz.x / 2 - 1,
                y: self.rect.pos.y + self.rect.sz.y,
            },
            sz: Vec2i { x: 2, y: 8 },
        }
    }
}

struct Blocker {
    rect: Rect,
    alive: bool,
}

impl Blocker {
    fn new(index: Vec2i) -> Self {
        assert!(index.x < 12, "{} is out of range 12", index.x);
        assert!(index.y < 2, "{} is out of range 2", index.y);
        Self {
            rect: Rect {
                pos: Vec2i {
                    x: 32 + 16 * index.x + 32 * (index.x / 4),
                    y: HEIGHT - 96 + 16 * index.y,
                },
                sz: Vec2i { x: 16, y: 16 },
            },
            alive: true,
        }
    }
}

fn main() {
    engine::eng::go::<Game>();
}

impl engine::eng::Game for Game {
    type Assets = Assets;
    type State = State;
    fn new() -> (State, Assets) {
        let spritesheet = Rc::new(Image::from_file(std::path::Path::new(
            "content/spritesheet.png",
        )));

        let assets = Assets { spritesheet };
        let state = State::new();
        (state, assets)
    }

    fn update(state: &mut State, _assets: &mut Assets, now_keys: &[bool], prev_keys: &[bool]) {
        use winit::event::VirtualKeyCode;

        // LEFT KEY
        if now_keys[VirtualKeyCode::Left as usize] {
            if state.vx > -1.0 {
                state.ax = -0.2;
            } else {
                state.ax = 0.0
            }
        }
        // RIGHT KEY
        if now_keys[VirtualKeyCode::Right as usize] {
            if state.vx < 1.0 {
                state.ax = 0.2;
            } else {
                state.ax = 0.0
            }
        }
        // BRAKING FORCE
        if !now_keys[VirtualKeyCode::Left as usize] && !now_keys[VirtualKeyCode::Right as usize] {
            if state.vx > 0.1 {
                state.ax = -0.1
            } else if state.vx < -0.1 {
                state.ax = 0.1
            } else {
                state.ax = 0.0
            }
        }

        if now_keys[VirtualKeyCode::Up as usize]
            && prev_keys[VirtualKeyCode::Up as usize]
            && state.shooting_timeout == 0
        {
            state.shooting_timeout = 20;
            state.player_bullets.push(Rect {
                pos: Vec2i {
                    x: state.player_sprite.shape.pos.x + state.player_sprite.shape.sz.x / 2 - 1,
                    y: state.player_sprite.shape.pos.y,
                },
                sz: Vec2i { x: 2, y: 8 },
            })
        }
    }

    fn render(state: &mut State, assets: &mut Assets, fb2d: &mut Image) {
        fb2d.clear((0, 0, 0, 255));

        // PLAYER MOVEMENT
        state.vx += state.ax;
        state.player_sprite.shape.move_by(state.vx as i32, 0);

        // PLAYER BOUNDS CHECK
        if state.player_sprite.shape.pos.x < 0 {
            state.player_sprite.shape.pos.x = 0
        }
        if state.player_sprite.shape.pos.x > WIDTH - state.player_sprite.shape.sz.x {
            state.player_sprite.shape.pos.x = WIDTH - state.player_sprite.shape.sz.x
        }

        // UPDATE PLAYER BULLETS
        if state.shooting_timeout > 0 {
            state.shooting_timeout -= 1
        }

        state.player_bullets.retain(|b| b.pos.y + b.sz.y > 0);

        for bullet in state.player_bullets.iter_mut() {
            bullet.pos.y -= 2*BULLET_VELO;
            fb2d.draw_rect(bullet, BLUE);
        }

        // UPDATE ENEMY BULLETS
        state.enemy_bullets.retain(|b| b.pos.y < HEIGHT);

        for bullet in state.enemy_bullets.iter_mut() {
            bullet.pos.y += BULLET_VELO;
            fb2d.draw_rect(bullet, RED);
        }


        // UPDATE PLAYER
        fb2d.bitblt(
            &assets.spritesheet,
            SS_PLAYER,
            state.player_sprite.shape.pos,
            false,
        );

        // UPDATE ENEMIES
        let left = state.enemies[0].rect.pos.x;
        let right = state.enemies.last().unwrap().rect.pos.x + 16;
        if left <= 16 || right >= WIDTH - 16 {
            state.evx *= -1
        }

        let mut rng = rand::thread_rng();

        let mut enemies_left = false;
        for enemy in state.enemies.iter_mut() {

            if enemy.alive {
                enemies_left = true;
            }

            let mut dead_bullets = vec![];

            for (i,player_bullet) in state.player_bullets.iter().enumerate() {
                if enemy.rect.contains_point(player_bullet.pos) && enemy.alive {
                    // play death animation
                    enemy.alive = false;
                    dead_bullets.push(i);
                }
            }

            for i in dead_bullets {
                state.player_bullets.remove(i);
            }

            enemy.rect.move_by(state.evx, 0);

            if rng.gen_range(0..600) == 0 && enemy.alive {
                state.enemy_bullets.push(enemy.shoot());
            }

            let temp: Rect = Rect {
                pos: Vec2i { x: 0, y: 16*enemy.style },
                sz: Vec2i { x: 16, y: 16 }
            };

            if enemy.alive {
                let speedup_factor = 7;
                // fb2d.bitblt(&assets.spritesheet, TEMP, enemy.rect.pos, false);
                fb2d.bitblt(
                    &assets.spritesheet,
                    enemy.sprite.play_animation(&speedup_factor),
                    enemy.rect.pos,
                    false,
                );
            }
        }

        if !enemies_left {
            state.game_over = 2;
            // win sequence
        }

        // UPDATE BLOCKERS
        for blocker in state.blockers.iter_mut() {

            let mut dead_player_bullets = vec![];

            for (i,player_bullet) in state.player_bullets.iter().enumerate() {
                if blocker.rect.contains_point(player_bullet.pos) && blocker.alive {
                    // play death animation
                    blocker.alive = false;
                    dead_player_bullets.push(i);
                }
            }

            for i in dead_player_bullets {
                state.player_bullets.remove(i);
            }

            let mut dead_enemy_bullets = vec![];

            for (i,enemy_bullet) in state.enemy_bullets.iter().enumerate() {
                if blocker.rect.contains_point(enemy_bullet.pos) && blocker.alive {
                    // play death animation
                    blocker.alive = false;
                    dead_enemy_bullets.push(i);
                }
            }

            for i in dead_enemy_bullets {
                state.enemy_bullets.remove(i);
            }

            if blocker.alive {
                fb2d.draw_rect(&blocker.rect, BLUE);
            }
        }

        // ENEMY BULLET & PLAYER COLLISION
        for enemy_bullet in state.enemy_bullets.iter() {
            if state.player.contains_point(enemy_bullet.pos) {
                if state.game_over == 0 {
                    state.game_over = 1;
                    // loss sequence
                }
            }
        }
    }
}
