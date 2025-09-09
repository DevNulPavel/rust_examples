use hw8_fifteen::puzzle::{Goal, GoalLast, SlidePuzzle15, a_star};
use peak_alloc::PeakAlloc;
use slint::Model;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;
use std::time::Instant;

slint::include_modules!();

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

pub fn main() -> Result<(), slint::PlatformError> {
    let main_window = Puzzle15Window::new()?;

    let state = Rc::new(RefCell::new(AppState {
        pieces: Rc::new(slint::VecModel::<Piece>::from(vec![Piece::default(); 15])),
        main_window: main_window.as_weak(),
        positions: vec![],
        auto_play_timer: Default::default(),
        solution: None,
        kick_animation_timer: Default::default(),
        speed_for_kick_animation: Default::default(),
        finished: false,
        blocked: false,
    }));
    state.borrow_mut().randomize();
    let (sp, m) = state.borrow().get_slide_puzzle();
    println!("gen: {sp:?}, m: {m}");
    main_window.set_pieces(state.borrow().pieces.clone().into());

    let state_copy = state.clone();
    // Обработчик клика
    main_window.on_piece_clicked(move |p| {
        if state_copy.borrow().finished || state_copy.borrow().blocked {
            return;
        }
        if !state_copy.borrow_mut().piece_clicked(p as i8) {
            let state_weak = Rc::downgrade(&state_copy);
            state_copy.borrow().kick_animation_timer.start(
                slint::TimerMode::Repeated,
                std::time::Duration::from_millis(16),
                move || {
                    if let Some(state) = state_weak.upgrade() {
                        state.borrow_mut().kick_animation();
                    }
                },
            );
        }
    });

    let state_copy = state.clone();
    main_window.on_reset(move || {
        state_copy.borrow().auto_play_timer.stop();
        state_copy.borrow_mut().blocked = false;
        state_copy.borrow_mut().solution = None;
        state_copy.borrow_mut().randomize();
        let (sp, m) = state_copy.borrow().get_slide_puzzle();
        println!("gen: {sp:?}, m: {m}");
    });

    let state_copy = state.clone();
    main_window.on_win(move || {
        if !(state_copy.borrow().finished || state_copy.borrow().blocked) {
            state_copy.borrow_mut().blocked = true;

            let start = Instant::now();
            let mut solution: Option<VecDeque<usize>> = None;
            let (sp, m) = state_copy.borrow().get_slide_puzzle();
            println!("win: {sp:?}, m: {m}");

            PEAK_ALLOC.reset_peak_usage();
            if let Some(s) = a_star(sp) {
                let c: VecDeque<usize> = s.iter().skip(1).map(|h| h.hole()).collect();
                let peak_mem = PEAK_ALLOC.peak_usage_as_gb();
                let m = state_copy.borrow().main_window.unwrap().get_moves();
                if m > 0 {
                    print!("{m} + ");
                }
                println!("{} - {c:?}", c.len());
                println!("{:?}, mem usage: {peak_mem} GiB", start.elapsed());
                solution = Some(c);
            }
            state_copy.borrow_mut().solution = solution;

            let state_weak = Rc::downgrade(&state_copy);
            state_copy.borrow().auto_play_timer.start(
                slint::TimerMode::Repeated,
                std::time::Duration::from_secs(1),
                move || {
                    if let Some(state) = state_weak.upgrade() {
                        state.borrow_mut().solution_step_play();
                    }
                },
            );
        }
    });

    main_window.run()
}

fn is_solvable(positions: &[i8]) -> bool {
    // Same source as the flutter's slide_puzzle:
    // https://www.cs.bham.ac.uk/~mdr/teaching/modules04/java2/TilesSolvability.html
    // This page seems to be no longer available, a copy can be found here:
    // https://horatiuvlad.com/unitbv/inteligenta_artificiala/2015/TilesSolvability.html
    //
    // https://unitbv.horatiuvlad.com
    // https://s3.eu-central-1.amazonaws.com/unitbv.horatiuvlad.com/facultate/inteligenta_artificiala/2015/TilesSolvability.html

    let mut inversions = 0;
    for x in 0..positions.len() - 1 {
        let v = positions[x];
        inversions += positions[x + 1..]
            .iter()
            .filter(|x| **x >= 0 && **x < v)
            .count();
    }
    //((blank on odd row from bottom) == (#inversions even))
    let blank_row = positions.iter().position(|x| *x == -1).unwrap() / 4;
    inversions % 2 != blank_row % 2
}

fn shuffle() -> Vec<i8> {
    let mut vec = ((-1)..15).collect::<Vec<i8>>();
    use rand::seq::SliceRandom;
    let mut rng = rand::rng();
    vec.shuffle(&mut rng);
    while !is_solvable(&vec) {
        vec.shuffle(&mut rng);
    }
    vec
}

struct AppState {
    pieces: Rc<slint::VecModel<Piece>>,
    main_window: slint::Weak<Puzzle15Window>,
    /// An array of 16 values which represent a 4x4 matrix containing the piece number in that
    /// position. -1 is no piece.
    positions: Vec<i8>,
    auto_play_timer: slint::Timer,
    solution: Option<VecDeque<usize>>,
    kick_animation_timer: slint::Timer,
    /// The speed in the x and y direction for the associated tile
    speed_for_kick_animation: [(f32, f32); 15],
    finished: bool,
    blocked: bool,
}

impl AppState {
    fn set_pieces_pos(&self, p: i8, pos: i8) {
        if p >= 0 {
            self.pieces.set_row_data(
                p as usize,
                Piece {
                    pos_y: (pos % 4) as _,
                    pos_x: (pos / 4) as _,
                    offset_x: 0.,
                    offset_y: 0.,
                },
            );
        }
    }

    /// Return `(SlidePuzzle15, manhattan)`
    fn get_slide_puzzle(&self) -> (SlidePuzzle15, usize) {
        let mut u: [u8; 16] = [0; 16];
        for (i, v) in self.positions.iter().enumerate() {
            u[i] = (*v + 1) as u8;
        }

        let sp = SlidePuzzle15::from_array(u.as_ref()).unwrap();
        let gl = GoalLast::new();
        let m = gl.manhattan(&sp);
        (sp, m)
    }

    fn randomize(&mut self) {
        self.positions = shuffle();
        for (i, p) in self.positions.iter().enumerate() {
            self.set_pieces_pos(*p, i as _);
        }
        self.main_window.unwrap().set_moves(0);
        self.apply_tiles_left();
    }

    fn apply_tiles_left(&mut self) {
        let left = 15
            - self
                .positions
                .iter()
                .enumerate()
                .filter(|(i, x)| *i as i8 == **x)
                .count();
        self.main_window.unwrap().set_tiles_left(left as _);
        self.finished = left == 0;
    }

    fn piece_clicked(&mut self, p: i8) -> bool {
        let piece = self.pieces.row_data(p as usize).unwrap_or_default();
        assert_eq!(self.positions[(piece.pos_x * 4 + piece.pos_y) as usize], p);

        // find the coordinate of the hole.
        let hole = self.positions.iter().position(|x| *x == -1).unwrap() as i8;
        let pos = (piece.pos_x * 4 + piece.pos_y) as i8;
        let sign = if pos > hole { -1 } else { 1 };
        if hole % 4 == piece.pos_y as i8 {
            self.slide(pos, sign * 4)
        } else if hole / 4 == piece.pos_x as i8 {
            self.slide(pos, sign)
        } else {
            self.speed_for_kick_animation[p as usize] = (
                if hole % 4 > piece.pos_y as i8 {
                    10.
                } else {
                    -10.
                },
                if hole / 4 > piece.pos_x as i8 {
                    10.
                } else {
                    -10.
                },
            );
            return false;
        };
        self.apply_tiles_left();
        if let Some(x) = self.main_window.upgrade() {
            x.set_moves(x.get_moves() + 1);
            if self.finished {
                println!("{}", self.main_window.unwrap().get_moves());
            }
        }
        true
    }

    fn slide(&mut self, pos: i8, offset: i8) {
        let mut swap = pos;
        while self.positions[pos as usize] != -1 {
            swap += offset;
            self.positions.swap(pos as usize, swap as usize);
            self.set_pieces_pos(self.positions[swap as usize] as _, swap);
        }
    }

    fn solution_step_play(&mut self) {
        if let Some(s) = self.solution.as_mut() {
            if let Some(p) = s.pop_front() {
                let d = self.positions[p];
                self.piece_clicked(d);
            } else if s.is_empty() {
                self.blocked = false;
            }
        }
    }

    /// Advance the kick animation
    fn kick_animation(&mut self) {
        /// update offset and speed, returns true if the animation is still running
        fn spring_animation(offset: &mut f32, speed: &mut f32) -> bool {
            const C: f32 = 0.3; // Constant = k/m
            const DAMP: f32 = 0.7;
            const EPS: f32 = 0.3;
            let acceleration = -*offset * C;
            *speed += acceleration;
            *speed *= DAMP;
            if *speed != 0. || *offset != 0. {
                *offset += *speed;
                if speed.abs() < EPS && offset.abs() < EPS {
                    *speed = 0.;
                    *offset = 0.;
                }
                true
            } else {
                false
            }
        }

        let mut has_animation = false;
        for idx in 0..15 {
            let mut p = self.pieces.row_data(idx).unwrap_or_default();
            let ax = spring_animation(&mut p.offset_x, &mut self.speed_for_kick_animation[idx].0);
            let ay = spring_animation(&mut p.offset_y, &mut self.speed_for_kick_animation[idx].1);
            if ax || ay {
                self.pieces.set_row_data(idx, p);
                has_animation = true;
            }
        }
        if !has_animation {
            self.kick_animation_timer.stop();
        }
    }
}

impl Debug for AppState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.positions)?;
        if self.finished {
            write!(f, " finished")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_first_last() {
        let mut vec = ((-1)..15).collect::<Vec<i8>>();
        let s = is_solvable(&vec);
        assert!(!s);

        vec.rotate_left(1);
        let s = is_solvable(&vec);
        assert!(s);
    }
}
