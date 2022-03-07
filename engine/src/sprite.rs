use crate::animations::AnimationState;
use crate::types::Rect;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Character {
    Mario,
    Luigi,
    SpaceInvader,
    SpaceInvaderEnemy1,
    SpaceInvaderEnemy2,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub enum Action {
    Walk,
    Die,
    Jump,
    Stand,
    Glide,
}

impl Action {
    #[allow(dead_code)]
    pub fn turn(&self) -> Self {
        use Action::*;
        match *self {
            Walk => Jump,
            Jump => Die,
            Die => Stand,
            Stand => Glide,
            Glide => Walk,
        }
    }
}

// pub trait DrawSpriteExt {
//     fn draw_sprite(&mut self, s: &Sprite, pos: Vec2i);
// }

// use crate::image::Image;
// impl DrawSpriteExt for Image {
//     fn draw_sprite(&mut self, s: &Sprite, pos: Vec2i) {
//         self.bitblt(&s.character, s.animation_state.current_frame(), pos);
//     }
// }

#[allow(dead_code)]
pub struct Sprite {
    pub character: Character,
    pub action: Action,
    pub animation_state: AnimationState,
    pub shape: Rect,
}

impl Sprite {
    #[allow(dead_code)]
    pub fn play_animation(&mut self, speedup_factor: &usize) -> Rect {
        self.animation_state.tick(speedup_factor) // you can use types to choose which animations to play
    }

    pub fn turn_action(&mut self) {
        self.action = self.action.turn();
    }
    pub fn set_animation(&mut self, animation_state: AnimationState) {
        self.action = animation_state.action;
        self.animation_state = animation_state;
    }

    #[allow(dead_code)]
    fn draw() {}
    #[allow(dead_code)]
    fn tick_animation() {}
}
