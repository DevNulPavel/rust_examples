use amethyst::ecs::{
    Component, 
    DenseVecStorage
};


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

/*impl Paddle {
    pub fn new(side: Side) -> Paddle {
        Paddle {
            velocity: 1.0,
            side,
            width: 1.0,
            height: 1.0,
        }
    }
}*/

impl Component for PaddleComponent {
    type Storage = DenseVecStorage<Self>;
}
////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Default)]
pub struct ScoreBoard {
    pub score_left: i32,
    pub score_right: i32,
}

/*impl ScoreBoard {
    pub fn new() -> ScoreBoard {
        ScoreBoard {
            score_left: 0,
            score_right: 0,
        }
    }
}*/
