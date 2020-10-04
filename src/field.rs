use wasm_bindgen::prelude::*;

#[wasm_bindgen]
#[repr(u32)]
#[derive(PartialEq, Clone, Copy)]
pub enum CellType {
    Empty = 0,
    Sand = 16777216 // 256 * 256 * 256
}

#[wasm_bindgen]
pub struct Field {
    pub width: usize,
    pub height: usize,
    values: Vec<CellType>,
}

#[wasm_bindgen]
pub struct FieldSimple {
    pub width: usize,
    pub height: usize,
    values: Vec<u32>,
}

#[wasm_bindgen]
impl FieldSimple {
    pub fn new(w: usize, h: usize) -> FieldSimple {
        let width = w;
        let height = h;
        let values = (0..(width * height)).into_iter().map(|idx| {
            let (x, y) = get_xy(width, height, idx);

            let rad = (x - 0.5).powf(2.) + (y - 0.5).powf(2.);
            if  rad <= (0.3 as f32).powf(2.0) && rad >= (0.2 as f32).powf(2.0)  { 256 * 256 * 256 } else { 0 }
        }).collect::<Vec<u32>>();

        FieldSimple { width, height, values }
    }

    pub fn bytes(&self) -> Vec<u8> {
        self.values
            .iter()
            .flat_map(|e: &u32| e.to_be_bytes().to_vec() )
            .collect()
    }
}

fn get_xy(w: usize, h: usize, idx: usize) -> (f32, f32) {
    let row = idx / w;
    let col = idx % w;

    (row as f32 / h as f32, col as f32 / w as f32)
}

#[wasm_bindgen]
impl Field {
    pub fn new(w: usize, h: usize) -> Field {
        let width = w;
        let height = h;
        let values = (0..(width * height)).into_iter().map(|idx| {
            let (x, y) = get_xy(width, height, idx);

            let rad = (x - 0.5).powf(2.) + (y - 0.5).powf(2.);
            if  rad <= (0.3 as f32).powf(2.0) && rad >= (0.2 as f32).powf(2.0)  { CellType::Sand } else { CellType::Empty }
        }).collect::<Vec<CellType>>();

        Field { width, height, values }
    }

    pub fn new_empty(w: usize, h: usize, value: CellType) -> Field {
        let width = w;
        let height = h;
        let values = (0..(width * height))
            .into_iter()
            .map(|_idx| value)
            .collect::<Vec<CellType>>();
        Field { width, height, values }
    }

    pub fn apply_force(&mut self, x: f32, y: f32, value: CellType, radius: usize) {
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

        if self.values[idx] != CellType::Empty {
            self.values[idx] = CellType::Empty
        } else {
            self.values[idx] = CellType::Sand;
        }
    } 

    pub fn step(&mut self) {
        
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
            .flat_map(|e: &CellType| (*e as u32).to_be_bytes().to_vec() )
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

                if v == CellType::Empty {
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
fn test_encoding() {
    let field = Field::new(32, 32);
    let field_simple = FieldSimple::new(32, 32);
    assert_eq!(field.bytes(), field_simple.bytes());
}