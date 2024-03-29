#[cfg(test)]
use crate::rules::Rules;

use crate::cell::CellType;

pub struct Field {
    pub width: usize,
    pub height: usize,
    values: Vec<CellType>,
}

impl Field {
    pub fn new(w: usize, h: usize, fill: impl Fn(usize) -> CellType) -> Field {
        let width = w;
        let height = h;
        let values = (0..(width * (height - 1)))
            .into_iter()
            .map(fill)
            .collect::<Vec<CellType>>();
        Field {
            width,
            height,
            values,
        }
    }

    pub fn bytes(&self) -> Vec<[f32; 4]> {
        let padding = (0..self.width)
            .map(|_| &CellType::Wall)
            .map(CellType::pack_rgba);
        let bytes = self.values.iter().map(CellType::pack_rgba);

        padding.chain(bytes).collect()
    }
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
    pub fn get_idx(&self, row: usize, col: usize) -> usize {
        row * self.width + col
    }

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

    fn step(&mut self, time_step: u32, rules: &Rules) {
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

            let shifted = rules.rules(nh);

            if shifted != nh {
                println!("gid={} ({},{}) {:?} -> {:?}", gid, row, col, nh, shifted);
            }

            new_values[idx] = shifted[(gid - 1) as usize];
        }
        self.values = new_values;
    }
}


#[cfg(test)]
impl std::fmt::Display for Field {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, " ")?;
        for i in 0..self.height() {
            write!(f, "{}", i)?;
        }
        writeln!(f)?;

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
            writeln!(f)?;
        }

        Ok(())
    }
}

#[cfg(test)]
fn get_xy(w: usize, h: usize, idx: usize) -> (f32, f32) {
    let row = idx / w;
    let col = idx % w;

    (row as f32 / h as f32, col as f32 / w as f32)
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
    let rules = Rules::new();

    let invariant = [Sand, Sand, Sand, Sand];
    assert_eq!(rules.rules(invariant), invariant);
}

#[test]
fn test_encode_nh() {
    use CellType::*;

    let field = Field::new(32, 32, |idx| {
        let (x, y) = get_xy(32, 32, idx);

        let rad = (x - 0.5).powf(2.) + (y - 0.5).powf(2.);
        if rad <= (0.3 as f32).powf(2.0) && rad >= (0.2 as f32).powf(2.0) {
            CellType::Sand
        } else {
            CellType::Empty
        }
    });

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

    let mut field = Field::new(2, 3, |_| Sand);

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

    let rules = Rules::new();
    let mut field = Field::new(4, 5, |_| Empty);

    field.togglerc(1, 1);
    field.togglerc(2, 2);

    println!("field{}", field);

    println!("nh (1, 1) {:?}", field.encodde_neighborhood(1, 1, 1));

    field.step(1, &rules);

    println!("field{}", field);

    assert_eq!(
        field.encodde_neighborhood(1, 1, 1),
        [Empty, Empty, Sand, Sand]
    )
}
