use lazy_static::lazy_static;
use pathfinding::prelude::astar;
use rand::prelude::{SliceRandom, ThreadRng};
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::num::NonZeroU8;

lazy_static! {
    static ref SUCCESSORS: Vec<Vec<usize>> = (0..4 * 4)
        .map(|idx| (0..4)
            .filter_map(|dir| match dir {
                0 if idx % 4 > 0 => Some(idx - 1),
                1 if idx >= 4 => Some(idx - 4),
                2 if idx % 4 < 4 - 1 => Some(idx + 1),
                3 if idx < 4 * 4 - 4 => Some(idx + 4),
                _ => None,
            })
            .collect::<Vec<_>>())
        .collect();
    static ref GOAL_FIRST: GoalFirst = GoalFirst;
    static ref GOAL_LAST: GoalLast = GoalLast;
}

pub trait Goal {
    /// Получить выигранную версию игры
    fn goal_solved(&self) -> SlidePuzzle15;

    /// [Tiles Game - Formula for determining solvability](https://s3.eu-central-1.amazonaws.com/unitbv.horatiuvlad.com/facultate/inteligenta_artificiala/2015/TilesSolvability.html)
    fn is_solvable(&self, puzzle: &SlidePuzzle15) -> bool;

    /// Проверить завершение игры
    fn is_solved(&self, puzzle: &SlidePuzzle15) -> bool;

    /// Дистанция до требуемой позиции (Manhattan Distance)
    fn distance(&self, puzzle: &SlidePuzzle15, idx: usize) -> Option<usize>;

    /// Manhattan Distance для всей игры (сумма всех позиций без дырки)
    fn manhattan(&self, puzzle: &SlidePuzzle15) -> usize {
        let mut d: usize = 0;
        for (i, v) in puzzle.dibs.iter().enumerate() {
            if v.is_some() {
                d += self.distance(puzzle, i).unwrap();
            }
        }
        d
    }

    /// Перемешать фишки
    fn shuffle(&self, puzzle: &mut SlidePuzzle15) {
        let mut rng = ThreadRng::default();
        loop {
            puzzle.dibs.shuffle(&mut rng);
            // Дырка должна быть
            puzzle.hole = puzzle.dibs.iter().position(|d| d.is_none()).unwrap() as u8;
            if self.is_solvable(puzzle) {
                break;
            }
        }
    }
}

/// Дырка первая
pub struct GoalFirst;

impl GoalFirst {
    pub fn new() -> Self {
        GoalFirst
    }
}

impl Goal for GoalFirst {
    fn goal_solved(&self) -> SlidePuzzle15 {
        let dibs = [
            None,
            NonZeroU8::new(1),
            NonZeroU8::new(2),
            NonZeroU8::new(3),
            NonZeroU8::new(4),
            NonZeroU8::new(5),
            NonZeroU8::new(6),
            NonZeroU8::new(7),
            NonZeroU8::new(8),
            NonZeroU8::new(9),
            NonZeroU8::new(10),
            NonZeroU8::new(11),
            NonZeroU8::new(12),
            NonZeroU8::new(13),
            NonZeroU8::new(14),
            NonZeroU8::new(15),
        ];

        SlidePuzzle15 {
            size: 4,
            hole: 0,
            dibs,
        }
    }

    fn is_solvable(&self, puzzle: &SlidePuzzle15) -> bool {
        let inversions = inversions(puzzle);

        // Решение когда дырка первая для любого размера
        if puzzle.size % 2 == 1 {
            inversions % 2 == 0
        } else {
            puzzle.y(puzzle.hole()) % 2 == inversions % 2
        }
    }

    fn is_solved(&self, puzzle: &SlidePuzzle15) -> bool {
        if puzzle.dibs[0].is_none() {
            let last_idx = puzzle.size() * puzzle.size() - 1;
            for i in 1..=last_idx {
                if puzzle.dibs[i] != NonZeroU8::new(i as u8) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    fn distance(&self, puzzle: &SlidePuzzle15, idx: usize) -> Option<usize> {
        if idx >= puzzle.dibs.len() {
            return None;
        }
        let v = puzzle.dibs[idx];
        match v {
            None => None,
            Some(n) => {
                let (correct_x, correct_y) = (puzzle.x(idx), puzzle.y(idx));
                let k = n.get() as usize;
                let (actual_x, actual_y) = (puzzle.x(k), puzzle.y(k));
                Some(actual_x.abs_diff(correct_x) + actual_y.abs_diff(correct_y))
            }
        }
    }
}

impl Default for GoalFirst {
    fn default() -> Self {
        Self::new()
    }
}

/// Дырка последняя
pub struct GoalLast;

impl GoalLast {
    pub fn new() -> Self {
        GoalLast
    }
}

impl Goal for GoalLast {
    fn goal_solved(&self) -> SlidePuzzle15 {
        let dibs = [
            NonZeroU8::new(1),
            NonZeroU8::new(2),
            NonZeroU8::new(3),
            NonZeroU8::new(4),
            NonZeroU8::new(5),
            NonZeroU8::new(6),
            NonZeroU8::new(7),
            NonZeroU8::new(8),
            NonZeroU8::new(9),
            NonZeroU8::new(10),
            NonZeroU8::new(11),
            NonZeroU8::new(12),
            NonZeroU8::new(13),
            NonZeroU8::new(14),
            NonZeroU8::new(15),
            None,
        ];

        SlidePuzzle15 {
            size: 4,
            hole: 15,
            dibs,
        }
    }

    fn is_solvable(&self, puzzle: &SlidePuzzle15) -> bool {
        // Для чётных (нужно для всех)
        // if puzzle.size_x != 4 || puzzle.size_y != 4 {
        //     return Err(SlidePuzzleError::IncorrectSize)
        // }

        let inversions = inversions(puzzle);

        // Решение когда дырка последняя для чётного размера
        // Как то надо проверить
        //((blank on odd row from bottom) == (#inversions even))
        let blank_row = puzzle.y(puzzle.hole());
        inversions % 2 != blank_row % 2
    }

    fn is_solved(&self, puzzle: &SlidePuzzle15) -> bool {
        let last_idx = puzzle.size() * puzzle.size() - 1;
        if puzzle.dibs[last_idx].is_none() {
            for i in 0..last_idx {
                if puzzle.dibs[i] != NonZeroU8::new((i + 1) as u8) {
                    return false;
                }
            }
            return true;
        }
        false
    }

    fn distance(&self, puzzle: &SlidePuzzle15, idx: usize) -> Option<usize> {
        if idx >= puzzle.dibs.len() {
            return None;
        }
        let v = puzzle.dibs[idx];
        match v {
            None => None,
            Some(n) => {
                let (correct_x, correct_y) = (puzzle.x(idx), puzzle.y(idx));
                let k = n.get() as usize - 1;
                let (actual_x, actual_y) = (puzzle.x(k), puzzle.y(k));
                Some(actual_x.abs_diff(correct_x) + actual_y.abs_diff(correct_y))
            }
        }
    }
}

impl Default for GoalLast {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, PartialEq, Hash)]
pub struct SlidePuzzle15 {
    size: u8,
    hole: u8,
    dibs: [Option<NonZeroU8>; 16],
}

#[derive(Clone, Debug, PartialEq)]
pub enum SlidePuzzleError {
    IncorrectSize,
    IncorrectDibs,
}

impl SlidePuzzle15 {
    pub fn from_array(source: &[u8]) -> Result<Self, SlidePuzzleError> {
        let n = source.len();

        let size: usize = match n {
            16 => 4,
            _ => return Err(SlidePuzzleError::IncorrectSize),
        };

        let theorem = (n - 1) * n / 2; // Теорема о сумме первых n натуральных чисел: S = N(N+1)/2

        let mut hole: usize = 0;
        let mut dibs = [None; 16];

        let mut sum: usize = 0;
        for (i, &v) in source.iter().filter(|&p| *p < (n as u8)).enumerate() {
            if v == 0_u8 {
                hole = i;
            }
            sum += v as usize;
            dibs[i] = NonZeroU8::new(v);
        }

        if sum != theorem {
            Err(SlidePuzzleError::IncorrectDibs)
        } else {
            Ok(SlidePuzzle15 {
                size: size as u8,
                hole: hole as u8,
                dibs,
            })
        }
    }

    #[inline]
    pub const fn size(&self) -> usize {
        self.size as usize
    }

    pub const fn hole(&self) -> usize {
        self.hole as usize
    }

    #[inline]
    pub const fn x(&self, pos: usize) -> usize {
        pos % self.size()
    }

    #[inline]
    pub const fn y(&self, pos: usize) -> usize {
        pos / self.size()
    }

    pub fn swap_hole(&mut self, idx: usize) {
        self.dibs.swap(self.hole as usize, idx);
        self.hole = idx as u8;
    }

    pub fn swap_hole_to_new(&self, idx: usize) -> Self {
        let mut r = self.clone();
        r.dibs.swap(self.hole(), idx);
        r.hole = idx as u8;
        r
    }
}

impl Display for SlidePuzzle15 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, d) in self.dibs.iter().enumerate() {
            if i % self.size() == 0 {
                writeln!(f)?;
            }
            if let Some(n) = d {
                write!(f, "{n:2} ")?;
            } else {
                write!(f, "   ")?;
            }
        }
        Ok(())
    }
}

impl Debug for SlidePuzzle15 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}*{}, {} [", self.size, self.size, self.hole)?;
        for (i, d) in self.dibs.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            if let Some(n) = d {
                write!(f, "{n:2}")?;
            } else {
                write!(f, " 0")?;
            }
        }
        write!(f, "]")
    }
}

/// Node for A* algorithm
#[derive(Clone)]
pub struct Node {
    puzzle: SlidePuzzle15,
}

impl Node {
    pub fn new(puzzle: SlidePuzzle15) -> Self {
        Node { puzzle }
    }

    pub fn manhattan(&self) -> usize {
        GOAL_LAST.manhattan(&self.puzzle)
    }

    pub fn is_solved(&self) -> bool {
        GOAL_LAST.is_solved(&self.puzzle)
    }

    fn successors(&self) -> impl Iterator<Item = (Node, usize)> + use<> {
        let r = self.clone();
        SUCCESSORS[self.puzzle.hole()]
            .iter()
            .map(move |&p| (Node::new(r.puzzle.swap_hole_to_new(p)), 1))
    }
}

impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.puzzle.hash(state);
    }
}

impl PartialEq<Self> for Node {
    fn eq(&self, other: &Self) -> bool {
        self.puzzle == other.puzzle
    }
}

impl Eq for Node {}

impl Debug for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.puzzle)
    }
}

fn inversions(puzzle: &SlidePuzzle15) -> usize {
    let limit = puzzle.size() * puzzle.size();
    let mut inversions = 0;
    for i in 0..limit {
        if let Some(c) = puzzle.dibs[i] {
            for j in i + 1..limit {
                if let Some(d) = puzzle.dibs[j]
                    && d < c
                {
                    inversions += 1;
                }
            }
        }
    }
    inversions
}

pub fn a_star(xp: SlidePuzzle15) -> Option<Vec<SlidePuzzle15>> {
    let s = Node::new(xp);
    if let Some((v, _)) = astar(&s, Node::successors, |p| p.manhattan(), Node::is_solved) {
        Some(v.iter().map(|n| n.puzzle.clone()).collect::<Vec<_>>())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use crate::puzzle::*;
    use pathfinding::prelude::astar;
    use std::time::Instant;

    #[test]
    fn test_a_star() {
        let start = Instant::now();

        let x: [u8; 16] = [12, 1, 10, 2, 7, 11, 4, 14, 5, 0, 9, 15, 8, 13, 6, 3];
        let xp = SlidePuzzle15::from_array(x.as_ref()).unwrap();
        println!("{xp}");

        let s = Node::new(xp);
        if let Some((v, m)) = astar(&s, Node::successors, |p| p.manhattan(), Node::is_solved) {
            println!("{m}");
            assert_eq!(m, 51);
            let c = v.iter().map(|n| n.puzzle.clone()).collect::<Vec<_>>();
            for (n, i) in c.iter().enumerate() {
                println!("{n:2} - {i:?}")
            }
            println!("{:?}", start.elapsed());
        } else {
            println!("no solution");
        }
    }

    #[test]
    fn test_size_of() {
        let m = size_of::<SlidePuzzle15>();
        assert_eq!(m, 18);
    }

    #[test]
    fn test_solvable() {
        let gf = GoalFirst::new();
        let gl = GoalLast::new();

        let first = gf.goal_solved();
        assert!(gf.is_solvable(&first));
        assert!(!gl.is_solvable(&first));

        let last = gl.goal_solved();
        assert!(gl.is_solvable(&last));

        // Example from: [Formula for determining solvability](https://s3.eu-central-1.amazonaws.com/unitbv.horatiuvlad.com/facultate/inteligenta_artificiala/2015/TilesSolvability.html)
        let x: [u8; 16] = [12, 1, 10, 2, 7, 11, 4, 14, 5, 0, 9, 15, 8, 13, 6, 3];
        let xp = SlidePuzzle15::from_array(x.as_ref()).unwrap();
        assert_eq!(49, inversions(&xp));
        assert!(gl.is_solvable(&xp));
    }

    #[test]
    fn test_swap_hole() {
        let gl = GoalLast::new();
        // Example from: [Formula for determining solvability](https://s3.eu-central-1.amazonaws.com/unitbv.horatiuvlad.com/facultate/inteligenta_artificiala/2015/TilesSolvability.html)
        let x: [u8; 16] = [12, 1, 10, 2, 7, 11, 4, 14, 5, 0, 9, 15, 8, 13, 6, 3];
        let mut xp = SlidePuzzle15::from_array(x.as_ref()).unwrap();
        println!("{xp}");
        xp.swap_hole(10);
        println!("{xp}");
        assert!(gl.is_solvable(&xp));
    }

    #[test]
    fn test_is_solved() {
        let gf = GoalFirst::new();
        let mut pf = gf.goal_solved();
        assert!(gf.is_solved(&pf));
        gf.shuffle(&mut pf);
        assert!(gf.is_solvable(&pf));
        assert!(!gf.is_solved(&pf));

        let gl = GoalLast::new();
        let mut pl = gl.goal_solved();
        assert!(gl.is_solved(&pl));
        gl.shuffle(&mut pl);
        assert!(gl.is_solvable(&pl));
        assert!(!gl.is_solved(&pl));
    }

    #[test]
    fn test_shuffled() {
        let gf = GoalFirst::new();
        let mut pf = gf.goal_solved();
        gf.shuffle(&mut pf);
        assert!(gf.is_solvable(&pf));

        let gl = GoalLast::new();
        let mut pl = gl.goal_solved();
        gl.shuffle(&mut pl);
        assert!(gl.is_solvable(&pl));
    }

    #[test]
    fn test_distance_for_goal_solved() {
        let gf = GoalFirst::new();

        let pf = gf.goal_solved();
        for i in 1..16_usize {
            assert_eq!(Some(0), gf.distance(&pf, i));
        }

        let gl = GoalLast::new();
        let pl = gl.goal_solved();
        for i in 0..15_usize {
            assert_eq!(Some(0), gl.distance(&pl, i));
        }
    }

    #[test]
    fn test_manhattan() {
        let gf = GoalFirst::new();

        let mut pf = gf.goal_solved();
        let m = gf.manhattan(&pf);
        assert_eq!(0, m);
        gf.shuffle(&mut pf);
        println!("{pf:?} - {}", gf.manhattan(&pf));

        let gl = GoalLast::new();
        let mut pl = gl.goal_solved();
        let m = gl.manhattan(&pl);
        assert_eq!(0, m);
        gl.shuffle(&mut pl);
        println!("{pl:?} - {}", gl.manhattan(&pl));
    }
}
