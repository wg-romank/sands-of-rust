use glsmrs::{
    texture::{ColorFormat, TextureSpec, UploadedTexture},
    Ctx,
};
use wasm_bindgen::prelude::*;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

#[wasm_bindgen]
#[repr(u32)]
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub enum CellType {
    Empty = 10,
    Water = 20,
    Sand = 30,
    Wall = 90,
}

pub struct Field {
    pub width: usize,
    pub height: usize,
    values: Vec<CellType>,
}

impl Into<f32> for CellType {
    fn into(self) -> f32 {
        (self as u32) as f32 / 255.
    }
}

#[cfg(test)]
fn get_xy(w: usize, h: usize, idx: usize) -> (f32, f32) {
    let row = idx / w;
    let col = idx % w;

    (row as f32 / h as f32, col as f32 / w as f32)
}

impl Field {
    #[cfg(test)]
    pub fn new(w: usize, h: usize) -> Field {
        let width = w;
        let height = h;
        let values = (0..(width * (height - 1)))
            .into_iter()
            .map(|idx| {
                let (x, y) = get_xy(width, height, idx);

                let rad = (x - 0.5).powf(2.) + (y - 0.5).powf(2.);
                if rad <= (0.3 as f32).powf(2.0) && rad >= (0.2 as f32).powf(2.0) {
                    CellType::Sand
                } else {
                    CellType::Empty
                }
            })
            .collect::<Vec<CellType>>();

        Field {
            width,
            height,
            values,
        }
    }

    pub fn new_empty(w: usize, h: usize, value: CellType) -> Field {
        let width = w;
        let height = h;
        let values = (0..(width * (height - 1)))
            .into_iter()
            .map(|_idx| value)
            .collect::<Vec<CellType>>();
        Field {
            width,
            height,
            values,
        }
    }

    pub fn bytes(&self) -> Vec<u8> {
        (0..self.width).flat_map(|_| (CellType::Wall as u32).to_le_bytes().to_vec()).chain(
            self.values
                .iter()
                .flat_map(|e: &CellType| (*e as u32).to_le_bytes().to_vec())
        ).collect()
    }
}

#[cfg(test)]
impl Field {
    pub fn get_idx(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }
}

use glsmrs::GL;
use CellType::*;

pub fn texture(ctx: &Ctx, arr: [[CellType; 4]; 9]) -> Result<UploadedTexture, String> {
    let useful_size = 2 * 9;
    let next_po2 = 32;
    let padding = (0..(next_po2 - useful_size)).flat_map(|_| (0 as u32).to_le_bytes().to_vec()).collect::<Vec<u8>>();
    log!("padding len {:?}", padding.len() / 4);
    let spec = TextureSpec::pixel(ColorFormat(GL::RGBA), [next_po2, 2]);

    let mut line1 = arr 
        .iter()
        .flat_map(|ar| {
            [ar[0], ar[1]]
                .iter()
                .flat_map(|e| (*e as u32).to_le_bytes().to_vec())
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<u8>>();
    line1.extend(&padding);

    let line2 = arr 
        .iter()
        .flat_map(|ar| {
            [ar[2], ar[3]]
                .iter()
                .flat_map(|e| (*e as u32).to_le_bytes().to_vec())
                .collect::<Vec<u8>>()
        })
        .collect::<Vec<u8>>();

    line1.extend(line2);
    line1.extend(&padding);

    spec.upload_u8(&ctx, &line1)
}

pub const PATTERNS: [[CellType; 4]; 9] = [
    [Sand, Empty, Empty, Empty], // i = 0
    [Sand, Sand, Sand, Empty],  // i = 2
    [Sand, Sand, Empty, Empty], // i = 4
    [Empty, Sand, Sand, Empty], // i = 6
    [Empty, Sand, Empty, Empty], // i = 8
    [Sand, Sand, Empty, Sand], // i != 10
    [Sand, Empty, Empty, Sand], // i != 12
    [Sand, Empty, Sand, Empty], // i != 14
    [Empty, Sand, Empty, Sand], // i == 16
];

pub const RULES: [[CellType; 4]; 9] = [
    [Empty, Empty, Sand, Empty],
    [Sand, Empty, Sand, Sand],
    [Empty, Empty, Sand, Sand],
    [Empty, Empty, Sand, Sand],
    [Empty, Empty, Empty, Sand],
    [Empty, Sand, Sand, Sand],
    [Empty, Empty, Sand, Sand],
    [Empty, Empty, Sand, Sand],
    [Empty, Empty, Sand, Sand],
];

#[cfg(test)]
fn rules(slice: [CellType; 4]) -> [CellType; 4] {
    let mut r = HashMap::new();
    for i in 0..PATTERNS.len() {
        r.insert(PATTERNS[i], RULES[i]);
    }
    r.get(&slice).unwrap_or(&slice).clone()
}

#[cfg(test)]
fn grid_idx(i: usize, j: usize, time_step: u32) -> u8 {
    let step_rounded = time_step % 2;
    let gid = match (i % 2, j % 2) {
        (0, 0) => 1,
        (0, 1) => 2,
        (1, 0) => 3,
        (1, 1) => 4,
        (_, _) => panic!("Invalid i={} j={}", i, j),
    };

    if step_rounded == 0 {
        return gid;
    } else {
        return match gid {
            1 => 4,
            4 => 1,
            2 => 3,
            3 => 2,
            o => o,
        };
    }
}

#[cfg(test)]
impl Field {
    fn height(&self) -> usize {
        self.height - 1
    }
    fn encodde_neighborhood(&self, gid: u8, row: usize, col: usize) -> [CellType; 4] {
        let (r, c) = (row as i32, col as i32);
        match gid {
            // * 2
            // 3 4
            1 => self.slice((r, c), (r, c + 1), (r + 1, c), (r + 1, c + 1)),
            // 1 *
            // 3 4
            2 => self.slice((r, c - 1), (r, c), (r + 1, c - 1), (r + 1, c)),
            // 1 2
            // * 4
            3 => self.slice((r - 1, c), (r - 1, c + 1), (r, c), (r, c + 1)),
            // 1 2
            // 3 *
            4 => self.slice((r - 1, c - 1), (r - 1, c), (r, c - 1), (r, c)),
            _ => panic!("GID is {}", gid),
        }
    }

    fn slice(
        &self,
        i1: (i32, i32),
        i2: (i32, i32),
        i3: (i32, i32),
        i4: (i32, i32),
    ) -> [CellType; 4] {
        [
            self.values[self.get_idx_clamp(i1.0, i1.1)],
            self.values[self.get_idx_clamp(i2.0, i2.1)],
            self.values[self.get_idx_clamp(i3.0, i3.1)],
            self.values[self.get_idx_clamp(i4.0, i4.1)],
        ]
    }

    fn get_idx_clamp(&self, row: i32, col: i32) -> usize {
        use std::cmp::max;
        use std::cmp::min;
        let row_u = min(max(0, row), (self.height() - 1) as i32) as usize;
        let col_u = min(max(0, col), (self.width - 1) as i32) as usize;

        self.get_idx(row_u, col_u)
    }

    fn togglerc(&mut self, row: usize, col: usize) {
        let idx = self.get_idx(row, col);

        if self.values[idx] != CellType::Empty {
            self.values[idx] = CellType::Empty
        } else {
            self.values[idx] = CellType::Sand;
        }
    }

    fn step(&mut self, time_step: u32) {
        let w = self.width;
        let h = self.height();
        let mut new_values = self.values.clone();
        for idx in 0..(w * h) {
            // skip some updates?
            // even grid on one time step
            // odd grid on another
            // for each cell determinte if it is on even grid or on odd?
            let row = idx / w;
            let col = idx % w;

            let gid = grid_idx(row, col, time_step);

            let nh = self.encodde_neighborhood(gid, row, col);

            let shifted = rules(nh);

            if shifted != nh {
                print!("gid={} ({},{}) {:?} -> {:?}\n", gid, row, col, nh, shifted);
            }

            new_values[idx] = shifted[(gid - 1) as usize];
        }
        self.values = new_values;
    }
}

#[cfg(test)]
use std::{fmt, collections::HashMap, hash::Hash};

#[cfg(test)]
impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " ")?;
        for i in 0..self.height() {
            write!(f, "{}", i)?;
        }
        writeln!(f, "")?;

        for i in 0..self.width {
            write!(f, "{}", i)?;
            for j in 0..self.height() {
                let idx = self.get_idx(i, j);
                let v = self.values[idx];

                if v == CellType::Empty {
                    write!(f, "o")?;
                } else {
                    write!(f, "x")?;
                };
            }
            writeln!(f, "")?;
        }

        Ok(())
    }
}

#[test]
fn test_grid_idx() {
    for i in 0..10 {
        let gid = grid_idx(0, 0, i);
        if i % 2 == 0 {
            assert_eq!(gid, 1);
        } else {
            assert_eq!(gid, 4);
        }
    }
}

#[test]
fn test_rules_invariant() {
    use CellType::*;

    let invariant = [Sand, Sand, Sand, Sand];
    assert_eq!(rules(invariant), invariant);
}

#[test]
fn test_encode_nh() {
    use CellType::*;

    let field = Field::new(32, 32);
    let row = 0;
    let col = 0;
    let gid = grid_idx(row, col, 0);

    assert_eq!(
        field.encodde_neighborhood(gid, row, col),
        [Empty, Empty, Empty, Empty]
    );
}

#[test]
fn test_encode_nh_small_field() {
    use CellType::*;

    let row = 0;
    let col = 0;

    let mut field = Field::new_empty(2, 3, Sand);

    field.togglerc(row, col);
    field.togglerc(row + 1, col + 1);

    let gid = grid_idx(row, col, 0);
    assert_eq!(
        field.encodde_neighborhood(gid, row, col),
        [Empty, Sand, Sand, Empty]
    );
}

#[test]
fn test_step() {
    use CellType::*;

    let mut field = Field::new_empty(4, 5, Empty);

    field.togglerc(1, 1);
    field.togglerc(2, 2);

    print!("field\n{}", field);

    print!("nh (1, 1) {:?}\n", field.encodde_neighborhood(1, 1, 1));

    field.step(1);

    print!("field\n{}", field);

    assert_eq!(
        field.encodde_neighborhood(1, 1, 1),
        [Empty, Empty, Sand, Sand]
    )
}

#[test]
fn understand_conversion() {
    use CellType::*;

    assert_eq!(Water as u32, 20);
    assert_eq!(Water as u32 as f32, 20.);
}
