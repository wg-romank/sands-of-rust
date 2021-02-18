use wasm_bindgen::prelude::*;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub struct Cell {
    value: u8,
    name: &'static str,
    color: &'static str,
}

impl Cell {
    const fn new(value: u8, name: &'static str, color: &'static str) -> Cell {
        Cell {value, name: name, color: color}
    }

    pub fn gen_shader() -> String {
        cells.iter().fold(String::new(), |acc, c| acc + format!("const float {} = {}.;\n", c.name, c.value).as_str())
    }
}

const cells: [Cell; 3] = [
    Cell::new(10, "EMPTY", "black"),
    Cell::new(20, "WATER", "black"),
    Cell::new(30, "SAND", "black"),
];

#[wasm_bindgen]
#[repr(u32)]
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum CellType {
    Empty = 10,
    Water = 20,
    Sand = 30,
}


#[wasm_bindgen]
pub struct Field {
    pub width: usize,
    pub height: usize,
    values: Vec<CellType>,
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
            .flat_map(|e: &CellType| (*e as u32).to_le_bytes().to_vec() )
            .collect()
    }
}

fn rules(slice: [CellType; 4]) -> [CellType; 4] {
    use CellType::*;
    match slice {
        [Sand, Empty,
         Empty, Empty] => [Empty, Empty,
                           Sand, Empty],
        [Sand, Sand,
         Sand, Empty] => [Sand, Empty,
                          Sand, Sand],
        [Sand, Sand,
         Empty, Empty] => [Empty, Empty,
                           Sand, Sand],
        [Empty, Sand,
         Sand, Empty] => [Empty, Empty,
                          Sand, Sand],
        [Empty, Sand,
         Empty, Empty] => [Empty, Empty,
                           Empty, Sand],
        [Sand, Sand,
         Empty, Sand] => [Empty, Sand,
                          Sand, Sand],
        [Sand, Empty,
         Empty, Sand] => [Empty, Empty,
                          Sand, Sand],
        [Sand, Empty,
         Sand, Empty] => [Empty, Empty,
                          Sand, Sand],
        [Empty, Sand,
         Empty, Sand] => [Empty, Empty,
                          Sand, Sand],
        otherwise => otherwise
    }
}

fn grid_idx(i: usize, j: usize, time_step: u32) -> u8 {
    let step_rounded = time_step % 2;
    let gid = match (i % 2, j % 2) {
        (0, 0) => 1,
        (0, 1) => 2,
        (1, 0) => 3,
        (1, 1) => 4,
        (_, _) => panic!("Invalid i={} j={}", i, j)
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
        }
    }
}

impl Field {
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

    fn slice(&self, i1: (i32, i32), i2: (i32, i32), i3: (i32, i32), i4: (i32, i32)) -> [CellType; 4] {
        [
            self.values[self.get_idx_clamp(i1.0, i1.1)],
            self.values[self.get_idx_clamp(i2.0, i2.1)],
            self.values[self.get_idx_clamp(i3.0, i3.1)],
            self.values[self.get_idx_clamp(i4.0, i4.1)],
        ]
    }

    fn get_idx_clamp(&self, row: i32, col: i32) -> usize {
        use std::cmp::min;
        use std::cmp::max;
        let row_u = min(max(0, row), (self.height - 1) as i32) as usize;
        let col_u = min(max(0, col), (self.width - 1) as i32) as usize;

        self.get_idx(row_u, col_u)
    }


    fn toggle(&mut self, x: f32, y: f32) {
        let (row, col) = self.get_rc_from_xy(x, y);
        self.togglerc(row, col);
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
        let h = self.height;
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

use std::fmt;

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, " ")?;
        for i in 0..self.height {
            write!(f, "{}", i)?;
        }
        writeln!(f, "")?;

        for i in 0..self.width {
            write!(f, "{}", i)?;
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

struct FieldSimple {
    pub width: usize,
    pub height: usize,
    values: Vec<u32>,
}

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
        [Empty, Empty, Empty, Empty]);
}

#[test]
fn test_encode_nh_small_field() {
    use CellType::*;

    let row = 0;
    let col = 0;

    let mut field = Field::new_empty(2, 2, Sand);

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

    let mut field = Field::new_empty(4, 4, Empty);

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
fn understand_modulus() {
    assert_eq!(0 % 2, 0);
    assert_eq!(1 % 2, 1);
    assert_eq!(2 % 2, 0);
    assert_eq!(3 % 2, 1);
}

#[test]
fn understand_conversion() {
    use CellType::*;

    assert_eq!(Water as u32, 2);
    assert_eq!(Water as u32 as f32, 2.);
}
