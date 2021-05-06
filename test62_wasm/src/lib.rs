mod utils;

use std::{
    fmt
};
use wasm_bindgen::{
    prelude::{
        *
    }
};
use js_sys::{
    Math::{
        random
    }
};
use utils::{
    set_panic_hook
};

///////////////////////////////////////////////////////////////////////////////////////////

// Выставляем иной аллокатор если надо
#[cfg(feature = "tiny_allocator")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

///////////////////////////////////////////////////////////////////////////////////////////

// Внешние проэкспортированные функции
#[wasm_bindgen]
extern {
    fn alert(s: &str);
}

///////////////////////////////////////////////////////////////////////////////////////////

macro_rules! log {
    ( $( $t:tt )* ) => {
        #[allow(unused_unsafe)]
        unsafe{
            web_sys::console::log_1(&format!( $( $t )* ).into());
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a> {
        #[allow(unused_unsafe)]
        unsafe{
            web_sys::console::time_with_label(name);
        }
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        #[allow(unused_unsafe)]
        unsafe{
            web_sys::console::time_end_with_label(self.name);
        }
    }
}

///////////////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen]
pub fn greet(text: &str) {
    set_panic_hook();

    #[allow(unused_unsafe)]
    unsafe {
        alert(text);
    }
}

///////////////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen]
#[repr(u8)] // Enum будет представлен в виде байта
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn random_cell() -> Cell {
        #[allow(unused_unsafe)]
        let val = unsafe{
            random() < 0.5
        };
        if val {
            Cell::Alive
        } else {
            Cell::Dead
        }
    }

    fn toggle(&mut self) {
        *self = match *self {
            Cell::Dead => Cell::Alive,
            Cell::Alive => Cell::Dead,
        };
    }
}

///////////////////////////////////////////////////////////////////////////////////////////

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

/// Public methods, exported to JavaScript.
#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        utils::set_panic_hook();

        let width = 128;
        let height = 128;

        // let mut rnd_gen = rand::random();

        let cells = (0..width * height)
            .map(|_| {
                Cell::random_cell()
            })
            .collect();

        log!("Universe created");

        Universe {
            width,
            height,
            cells,
        }
    }

    pub fn get_width(&self) -> u32 {
        self.width
    }

    pub fn get_height(&self) -> u32 {
        self.height
    }

    pub fn get_cells_ptr(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");

        let mut next = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let idx = self.get_index(row, col);
                let cell = self.cells[idx];
                let live_neighbors = self.live_neighbor_count(row, col);

                let next_cell = match (cell, live_neighbors) {
                    // Rule 1: Any live cell with fewer than two live neighbours
                    // dies, as if caused by underpopulation.
                    (Cell::Alive, x) if x < 2 => {
                        Cell::Dead
                    },

                    // Rule 2: Any live cell with two or three live neighbours
                    // lives on to the next generation.
                    (Cell::Alive, 2) | (Cell::Alive, 3) => {
                        Cell::Alive
                    },

                    // Rule 3: Any live cell with more than three live
                    // neighbours dies, as if by overpopulation.
                    (Cell::Alive, x) if x > 3 => {
                        Cell::Dead
                    },

                    // Rule 4: Any dead cell with exactly three live neighbours
                    // becomes a live cell, as if by reproduction.
                    (Cell::Dead, 3) => {
                        Cell::Alive
                    },
                    
                    // All other cells remain in the same state.
                    (otherwise, _) => {
                        otherwise
                    },
                };

                next[idx] = next_cell;
            }
        }

        self.cells = next;
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        let idx = self.get_index(row, column);
        self.cells[idx].toggle();
    }

    pub fn reset_game(&mut self) {
        self
            .cells
            .iter_mut()
            .for_each(|val|{
                *val = Cell::random_cell();
            })
    }
}

impl Universe{
    fn live_neighbor_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_row == 0 && delta_col == 0 {
                    continue;
                }

                let neighbor_row = (row + delta_row) % self.height;
                let neighbor_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbor_row, neighbor_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    /// Set the width of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        self.cells = (0..width * self.height)
            .map(|_i| Cell::Dead)
            .collect();
    }

    /// Set the height of the universe.
    ///
    /// Resets all cells to the dead state.
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        self.cells = (0..self.width * height)
            .map(|_i| Cell::Dead)
            .collect();
    }

    /// Get the dead and alive values of the entire universe.
    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let chunks_iter = self
            .cells
            .as_slice()
            .chunks(self.width as usize);
        for line in chunks_iter {
            for &cell in line {
                let symbol = if cell == Cell::Dead { 
                    '◻' 
                } else { 
                    '◼' 
                };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }

        Ok(())
    }
}