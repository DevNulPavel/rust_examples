use amethyst::ecs::{
    //Entity,
    Component, 
    DenseVecStorage
};
use crate::constants::*;

pub struct BallComponent {
    pub velocity: [f32; 2],
    pub radius: f32,
}

impl Component for BallComponent {
    type Storage = DenseVecStorage<Self>;
}
////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(PartialEq, Eq)]
pub enum Side {
    Left,
    Right,
}

pub struct PaddleComponent {
    pub velocity: f32,
    pub side: Side,
    pub width: f32,
    pub height: f32,
}

impl PaddleComponent {
    pub fn new(side: Side) -> PaddleComponent {
        PaddleComponent {
            velocity: PADDLE_VELOCITY,
            side,
            width: PADDLE_WIDTH,
            height: PADDLE_HEIGHT,
        }
    }
}

impl Component for PaddleComponent {
    type Storage = DenseVecStorage<Self>;
}
////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct ScoreBoard {
    pub score_left: i32,
    pub score_right: i32,
}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct BounceCountComponent{
    pub count: u32
}

impl Component for BounceCountComponent{
    type Storage = DenseVecStorage<Self>;
}

////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct PointerComponent{
}

impl Component for PointerComponent{
    type Storage = DenseVecStorage<Self>;
}

////////////////////////////////////////////////////////////////////////////////////////////////

/*pub struct CameraEntity{
    pub cam: Entity
}*/