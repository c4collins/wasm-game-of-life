use std::fmt;

mod utils;

use fixedbitset::FixedBitSet;
use js_sys;
use web_sys::console;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

enum UniverseContents {
    Empty = 0,
    Random = 1,
    Lines = 2,
    Spaceship = 3,
}

fn build_cells(bitset: FixedBitSet, width: u32, height: u32, build_type:UniverseContents) -> FixedBitSet {
    let mut new_cells = bitset.clone();
    match build_type {
        UniverseContents::Spaceship => {
            // xxxxo
            // xooox
            // xoooo
            // oxoox
            console::log_1(&"building small spaceship".into());
            for i in 0..width * height {
                new_cells.set(
                    i as usize,
                    i == width + 0
                        || i == width + 1
                        || i == width + 2
                        || i == width + 3
                        || i == 2 * width
                        || i == 2 * width + 4
                        || i == 3 * width
                        || i == 4 * width + 1
                        || i == 4 * width + 4,
                );
            }
        }
        UniverseContents::Random => {
            console::log_1(&"building random universe".into());
            for i in 0..width * height {
                new_cells.set(i as usize, js_sys::Math::random() > 0.5);
            }
        }
        UniverseContents::Lines => {
            console::log_1(&"building mod&mod7 universe".into());
            for i in 0..width * height {
                new_cells.set(i as usize, i % 2 == 0 || i % 7 == 0);
            }
        }
        _ => {
            console::log_1(&"building empty universe".into());
            for i in 0..width * height {
                new_cells.set(i as usize, false);
            }
        }
    };
    new_cells
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        let width = 64;
        let height = 64;

        let size = (width * height) as usize;
        let bitset = FixedBitSet::with_capacity(size);

        Universe {
            width,
            height,
            cells: build_cells(bitset, width, height, UniverseContents::Random),
            // cells: build_cells(bitset, width, height, UniverseContents::Lines),
            // cells: build_cells(bitset, width, height, UniverseContents::Spaceship),
        }
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
                let live_neighbours = self.live_neighbour_count(row, col);

                next.set(
                    idx,
                    match (cell, live_neighbours) {
                        // Rule 1: Any live cell with fewer than two neighbours dies
                        (true, x) if x < 2 => false,
                        // Rule 2: Any live cell with 2 or 3 neighbours lives on
                        (true, 2) | (true, 3) => true,
                        // Rule 3: Any live cell with more than 3 neighbours dies
                        (true, x) if x > 3 => false,
                        // Rule 4: Any dead cell with exactly 3 live neighbours becomes alive
                        (false, 3) => true,
                        // Everything else stays as is (i.e. dead)
                        (otherwise, _) => otherwise,
                    },
                );
            }
        }
        self.cells = next;
    }

    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn set_width(&mut self, width: u32) {
        self.width = width;
        let bitset = FixedBitSet::with_capacity((width * self.height) as usize);
        self.cells = build_cells(bitset, width, self.height, UniverseContents::Empty);
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn set_height(&mut self, height: u32) {
        self.height = height;
        let bitset = FixedBitSet::with_capacity((self.width * height) as usize);
        self.cells = build_cells(bitset, self.width, height, UniverseContents::Empty);
    }
    pub fn cells(&self) -> *const u32 {
        self.cells.as_slice().as_ptr()
    }
}

impl Universe {
    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells.toggle(idx);
        }
    }
    pub fn get_cells(&self) -> &[u32] {
        &self.cells.as_slice()
    }

    fn live_neighbour_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_col == 0 && delta_row == 0 {
                    continue;
                }
                let neighbour_row = (row + delta_row) % self.height;
                let neightbour_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbour_row, neightbour_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let symbol = if cell == 0 { '◻' } else { '◼' };
                write!(f, "{}", symbol)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
