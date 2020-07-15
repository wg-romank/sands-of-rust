use wasm_bindgen::prelude::*;
use web_sys;

#[wasm_bindgen]
pub struct Field {
    pub width: usize,
    pub height: usize,
    values: Vec<u32>
}

fn get_xy(w: usize, h: usize, idx: usize) -> (f32, f32) {
    let row = idx / w;
    let col = idx % w;

    (row as f32 / h as f32, col as f32 / w as f32)
}

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
impl Field {
    pub fn new(w: usize, h: usize) -> Field {
        let width = w;
        let height = h;
        let values: Vec<u32> = (0..(width * height)).into_iter().map(|idx| {
            let (x, y) = get_xy(width, height, idx);

            let rad = (x - 0.5).powf(2.) + (y - 0.5).powf(2.);
            if  rad <= (0.3 as f32).powf(2.0) && rad >= (0.2 as f32).powf(2.0)  { 256 * 256 * 256 } else { 0 }
        }).collect();

        Field { width, height, values }
    }

    pub fn new_empty(w: usize, h: usize, value: f32) -> Field {
        let width = w;
        let height = h;
        let values: Vec<u32> = (0..(width * height)).into_iter().map(|_idx| { value as u32 }).collect();
        Field { width, height, values }
    }

    pub fn apply_force(&mut self, x: f32, y: f32, value: u32, radius: usize) {
        let (row, col) = self.get_rc_from_xy(x, y);

        for i in 0..(2 * radius) {
            let tmp = radius.pow(2) - (i - radius).pow(2);
            let limit = (tmp as f32).powf(0.5).floor() as usize;

            for j in 0..(2 * limit) {
                let r = row + (i - radius);
                let c = col + (j - limit);

                let idx = self.get_idx(r, c);
                if idx > 0 && idx < self.width * self.height {
                    self.values[idx] = value;
                }
            }
        }
    }

    pub fn toggle(&mut self, x: f32, y: f32) {
        let (row, col) = self.get_rc_from_xy(x, y);
        let idx = self.get_idx(row, col);

        if self.values[idx] != 0 {
            self.values[idx] = 0
        } else {
            self.values[idx] = 1;
        }
    } 
}

impl Field {
    pub fn get_idx(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }

    pub fn get_rc_from_xy(&self, x: f32, y: f32) -> (usize, usize) {
        let row = (x * self.width as f32) as usize;
        let col = (y * self.height as f32) as usize;

        (row, col)
    }

    pub fn xy(&self, idx: usize) -> (f32, f32) {
        get_xy(self.width, self.height, idx)
    }

    pub fn dx(&self) -> f32 { 1. / self.height as f32 }

    pub fn dy(&self) -> f32 { 1. / self.width as f32 }

    pub fn bytes(&self) -> Vec<u8> {
        self.values
            .iter()
            .flat_map(|e: &u32| e.to_be_bytes().to_vec() )
            .collect()
    }
}

use std::fmt;

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in 0..self.width {
            for j in 0..self.height {
                let idx = self.get_idx(i, j);
                let v = self.values[idx];

                if v == 0 {
                    write!(f, "o")?;
                } else {
                    write!(f, "x")?;
                };
            }
            writeln!(f, "")?;
        };

        Ok(())
    }
}

#[test]
fn init() {
    let field = Field::new(32, 32);
    print!("{}", field.to_string());
    // panic!("woah");
}