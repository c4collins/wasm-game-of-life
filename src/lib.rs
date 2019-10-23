// Standard
use std::fmt;
// External
use fixedbitset::FixedBitSet;
// External (WASM)
use js_sys;
use wasm_bindgen::prelude::*;
use web_sys::console;
// Internal
mod utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

macro_rules! log {
    ( $( $t:tt )* ) => {
        console::log_1(&format!($($t)*).into());
    }
}

enum UniverseContents {
    Empty = 0,
    Random = 1,
    Lines = 2,
    Spaceship = 3,
}

pub enum UniverseObjects {
    None = 0,
    Spaceship = 1,
    Pulsar = 2,
    Glider = 3,
}

fn build_cells(
    bitset: FixedBitSet,
    width: u32,
    height: u32,
    build_type: UniverseContents,
) -> FixedBitSet {
    let mut new_cells = bitset.clone();
    match build_type {
        UniverseContents::Spaceship => {
            // xxxxo
            // xooox
            // xoooo
            // oxoox
            log!("building small spaceship");
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
            log!("building random universe");
            for i in 0..width * height {
                new_cells.set(i as usize, js_sys::Math::random() > 0.5);
            }
        }
        UniverseContents::Lines => {
            log!("building i%2 & i%7 universe");
            for i in 0..width * height {
                new_cells.set(i as usize, i % 2 == 0 || i % 7 == 0);
            }
        }
        _ => {
            log!("building empty universe");
            for i in 0..width * height {
                new_cells.set(i as usize, false);
            }
        }
    };
    new_cells
}

#[wasm_bindgen]
#[derive(Clone)]
pub struct Universe {
    width: u32,
    height: u32,
    cells: FixedBitSet,
}

#[wasm_bindgen]
impl Universe {
    pub fn new() -> Universe {
        // Setup panic logging
        utils::set_panic_hook();

        let width = 80;
        let height = 64;

        let size = (width * height) as usize;
        let bitset = FixedBitSet::with_capacity(size);

        Universe {
            width,
            height,
            // cells: build_cells(bitset, width, height, UniverseContents::Random),
            cells: build_cells(bitset, width, height, UniverseContents::Lines),
            // cells: build_cells(bitset, width, height, UniverseContents::Spaceship),
        }
    }

    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn toggle_cell(&mut self, row: u32, column: u32) {
        self.set_cells(&[(row, column)]);
    }

    pub fn create_object(&mut self, type_name: &str, row: u32, col: u32) {
        let object_type = match type_name {
            "glider" => UniverseObjects::Glider,
            "spaceship" => UniverseObjects::Spaceship,
            "pulsar" => UniverseObjects::Pulsar,
            &_ => UniverseObjects::None,
        };
        self.create(object_type, row, col);
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
                        // Rule 3: Any live cell with more than 3 neighbours dies
                        (true, x) if x < 2 || x > 3 => {
                            // log!("{} is dying", idx);
                            false
                        }
                        // Rule 2: Any live cell with 2 or 3 neighbours lives on
                        // (true, 2) | (true, 3) => true,
                        // Rule 3: Any live cell with more than 3 neighbours dies
                        // (true, x) if x > 3 => false,
                        // Rule 4: Any dead cell with exactly 3 live neighbours becomes alive
                        (false, 3) => {
                            // log!("{} is coming to life", idx);
                            true
                        }
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
    pub fn clear_cells(&mut self) {
        let bitset = FixedBitSet::with_capacity((self.width * self.height) as usize);
        self.cells = build_cells(bitset, self.width, self.height, UniverseContents::Empty);
    }
    pub fn randomize_cells(&mut self) {
        let bitset = FixedBitSet::with_capacity((self.width * self.height) as usize);
        self.cells = build_cells(bitset, self.width, self.height, UniverseContents::Random);
    }

    fn get_index(&self, row: u32, column: u32) -> usize {
        (row * self.width + column) as usize
    }

    fn live_neighbour_count(&self, row: u32, column: u32) -> u8 {
        let mut count = 0;
        for delta_row in [self.height - 1, 0, 1].iter().cloned() {
            for delta_col in [self.width - 1, 0, 1].iter().cloned() {
                if delta_col == 0 && delta_row == 0 {
                    continue;
                }
                let neighbour_row = (row + delta_row) % self.height;
                let neighbour_col = (column + delta_col) % self.width;
                let idx = self.get_index(neighbour_row, neighbour_col);
                count += self.cells[idx] as u8;
            }
        }
        count
    }
}

impl Universe {
    fn create(&mut self, type_name: UniverseObjects, row: u32, col: u32) {
        match type_name {
            UniverseObjects::Pulsar => self.create_pulsar(row, col),
            UniverseObjects::Spaceship => self.create_spaceship(row, col),
            UniverseObjects::Glider => self.create_glider(row, col),
            _ => self.set_cells(&[(row, col)]),
        }
    }

    fn row_add(self, row: u32, to_add: u32) -> u32 {
        (row + to_add) % self.height
    }
    fn col_add(self, col: u32, to_add: u32) -> u32 {
        (col + to_add) % self.width
    }

    fn create_glider(&mut self, row: u32, col: u32) {
        self.set_cells(&[
            (row, col),
            (row, self.clone().col_add(col, 1)),
            (row, self.clone().col_add(col, 2)),
            (self.clone().row_add(row, 1), col),
            (self.clone().row_add(row, 2), self.clone().col_add(col, 1)),
        ]);
    }

    fn create_pulsar_horizontal_row(self, row: u32, col: u32) -> Vec<(u32, u32)> {
        vec![
            (row, self.clone().col_add(col, 2)),
            (row, self.clone().col_add(col, 3)),
            (row, self.clone().col_add(col, 4)),
            (row, self.clone().col_add(col, 8)),
            (row, self.clone().col_add(col, 9)),
            (row, self.clone().col_add(col, 10)),
        ]
    }
    fn create_pulsar_vertical_row(self, row: u32, col: u32) -> Vec<(u32, u32)> {
        vec![
            (row, col),
            (row, self.clone().col_add(col, 5)),
            (row, self.clone().col_add(col, 7)),
            (row, self.clone().col_add(col, 12)),
        ]
    }

    fn create_pulsar(&mut self, row: u32, col: u32) {
        let row_0 = self.clone().create_pulsar_horizontal_row(row, col);
        // 1
        let mut row_2 = self
            .clone()
            .create_pulsar_vertical_row(self.clone().row_add(row, 2), col);
        let mut row_3 = self
            .clone()
            .create_pulsar_vertical_row(self.clone().row_add(row, 3), col);
        let mut row_4 = self
            .clone()
            .create_pulsar_vertical_row(self.clone().row_add(row, 4), col);
        let mut row_5 = self
            .clone()
            .create_pulsar_horizontal_row(self.clone().row_add(row, 5), col);
        // 6
        let mut row_7 = self
            .clone()
            .create_pulsar_horizontal_row(self.clone().row_add(row, 7), col);
        let mut row_8 = self
            .clone()
            .create_pulsar_vertical_row(self.clone().row_add(row, 8), col);
        let mut row_9 = self
            .clone()
            .create_pulsar_vertical_row(self.clone().row_add(row, 9), col);
        let mut row_10 = self
            .clone()
            .create_pulsar_vertical_row(self.clone().row_add(row, 10), col);
        // 11
        let mut row_12 = self
            .clone()
            .create_pulsar_horizontal_row(self.clone().row_add(row, 12), col);
        let mut cells = row_0;
        cells.append(&mut row_2);
        cells.append(&mut row_3);
        cells.append(&mut row_4);
        cells.append(&mut row_5);
        cells.append(&mut row_7);
        cells.append(&mut row_8);
        cells.append(&mut row_9);
        cells.append(&mut row_10);
        cells.append(&mut row_12);

        self.set_cells(&cells);
    }

    fn create_spaceship(&mut self, row: u32, col: u32) {
        // TODO :: Not used, no code; can get called by create_object, but nothing triggers that match
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
