use rand::Rng;
use std::rc::Rc;
use winit;

use engine;
use engine::types::*;

const PLAYER_WIDTH: i32 = 32;
const PLAYER_HEIGHT: i32 = 16;
pub const WIDTH: i32 = 320;
pub const HEIGHT: i32 = 320;

const BULLET_VELO: i32 = 1;

const RED: Color = (181,49,32,255);
const BLUE: Color = (74, 206, 222, 255);

const TEMP: Rect = Rect {
    pos: Vec2i { x: 0, y: 0 },
    sz: Vec2i { x: 16, y: 16 }
};

const SS_PLAYER: Rect = Rect {
    pos: Vec2i { x: 32, y: 16 },
    sz: Vec2i { x: 32, y: 16 }
};

struct Assets {
    spritesheet: Rc<Image>
}

struct State {
    player: Rect,
    player_bullets: Vec<Rect>,
    vx: f32,
    ax: f32,

    enemies: Vec<Enemy>,
    enemy_bullets: Vec<Rect>,
    evx: i32,

    blockers: Vec<Blocker>,
    shooting_timeout: u8
}

impl State {
    pub fn new() -> Self {
        // SPRITES
        let player = Rect {
            pos: Vec2i {
                x: WIDTH / 2 - PLAYER_WIDTH / 2,
                y: HEIGHT - PLAYER_HEIGHT * 3,
            },
            sz: Vec2i {
                x: PLAYER_WIDTH,
                y: PLAYER_HEIGHT,
            },
        };

        let mut enemies = vec![];

        for y in 0..2 {
            for x in 0..8 {
                enemies.push(Enemy::new(x % 2 + y, Vec2i{x,y}));
            }
        }
        
        let mut blockers = vec![];
        for y in 0..2 {
            for x in 0..12 {
                blockers.push(Blocker::new(Vec2i{x, y}));
            }
        }

        State {
            player,
            player_bullets: vec![],
            vx: 0.0,
            ax: 0.0,

            enemies,
            enemy_bullets: vec![],
            evx: 1,

            blockers,
            shooting_timeout: 0,
        }
    }
}

struct Game {}

struct Enemy {
    style: i32,
    rect: Rect,
    alive: bool
}

impl Enemy {
    pub fn new(style: i32, index: Vec2i) -> Self {
        assert!(index.x < 8, "{} is out of range 8", index.x);
        assert!(index.y < 2, "{} is out of range 2", index.y);
        Self {
            style,
            rect: Rect {
                pos: Vec2i {
                    x: 64 + 16*index.x + (16 * index.x/2),
                    y: 32 + 32*index.y
                },
                sz: Vec2i { x: 16, y: 16 }
            },
            alive: true
        }
    }

    pub fn shoot(&self) -> Rect {
        Rect { 
            pos: Vec2i { 
                x: self.rect.pos.x + self.rect.sz.x/2 - 1, 
                y: self.rect.pos.y + self.rect.sz.y 
            }, 
            sz: Vec2i { x: 2, y: 8 }
        }
    }
}

struct Blocker {
    rect: Rect,
    alive: bool
}

impl Blocker {
    fn new(index: Vec2i) -> Self {
        assert!(index.x < 12, "{} is out of range 12", index.x);
        assert!(index.y < 2, "{} is out of range 2", index.y);
        Self {
            rect: Rect {
                pos: Vec2i {
                    x: 32 + 16*index.x + 32*(index.x/4),
                    y: HEIGHT - 96 + 16*index.y
                },
                sz: Vec2i { x: 16, y: 16 }
            },
            alive: true
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
        if !now_keys[VirtualKeyCode::Left as usize] && !now_keys[VirtualKeyCode::Right as usize]{
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
            state.player_bullets.push(
                Rect { 
                    pos: Vec2i { 
                        x: state.player.pos.x + state.player.sz.x/2 - 1, 
                        y: state.player.pos.y
                    }, 
                    sz: Vec2i { x: 2, y: 8 }
                }
            )
        }
    }

    fn render(state: &mut State, assets: &mut Assets, fb2d: &mut Image) {

        fb2d.clear((0,0,0,255));

        // PLAYER MOVEMENT
        state.vx += state.ax;
        state.player.move_by(state.vx as i32, 0);


        // PLAYER BOUNDS CHECK
        if state.player.pos.x < 0 {
            state.player.pos.x = 0
        }
        if state.player.pos.x > WIDTH - state.player.sz.x {
            state.player.pos.x = WIDTH - state.player.sz.x
        }

        // UPDATE PLAYER BULLETS
        if state.shooting_timeout > 0 { state.shooting_timeout -= 1 }

        state.player_bullets.retain(|b| b.pos.y + b.sz.y > 0);

        for bullet in state.player_bullets.iter_mut() {
            bullet.pos.y -= BULLET_VELO;
            fb2d.draw_rect(bullet, BLUE);
        }

        // UPDATE ENEMY BULLETS
        state.enemy_bullets.retain(|b| b.pos.y < HEIGHT);

        for bullet in state.enemy_bullets.iter_mut() {
            bullet.pos.y += BULLET_VELO;
            fb2d.draw_rect(bullet, RED);
        }
        

        // COLLISION CHECKS
        // player_bullet & enemy
        // player_bullet & blocker
        // enemy_bullet & player
        // enemy_bullet & blocker


        // UPDATE EVERYTHING
        fb2d.bitblt(
            &assets.spritesheet,
            SS_PLAYER,
            state.player.pos, 
            false
        );

        // UPDATE ENEMIES
        let left = state.enemies[0].rect.pos.x;
        let right = state.enemies.last().unwrap().rect.pos.x + 16;
        if left <= 16 || right >= WIDTH - 16 { state.evx *= -1 }

        let mut rng = rand::thread_rng();

        for enemy in state.enemies.iter_mut() {
            enemy.rect.move_by(state.evx, 0);

            if rng.gen_range(0..600) == 0 {
                state.enemy_bullets.push(enemy.shoot());
            }

            if enemy.alive {
                fb2d.bitblt(
                    &assets.spritesheet, 
                    TEMP,
                    enemy.rect.pos, 
                    false
                );
            }
        }

        // UPDATE BLOCKERS
        for blocker in state.blockers.iter() {
            fb2d.draw_rect(&blocker.rect, BLUE);
        }
    }
}
