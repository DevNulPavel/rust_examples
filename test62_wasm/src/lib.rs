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
        let width = 128;
        let height = 128;

        // let mut rnd_gen = rand::random();

        let cells = (0..width * height)
            .map(|_| {
                #[allow(unused_unsafe)]
                let val = unsafe{
                    random() < 0.5
                };
                if val {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe {
            width,
            height,
            cells,
        }
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
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

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn tick(&mut self) {
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