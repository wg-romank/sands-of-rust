use wasm_bindgen::prelude::*;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub struct Field {
    pub width: usize,
    pub height: usize,
    values: Vec<f32>,
}

fn get_xy(w: usize, h: usize, idx: usize) -> (f32, f32) {
    let row = idx / w;
    let col = idx % w;

    (row as f32 / h as f32, col as f32 / w as f32)
}

impl Field {
    pub fn new_empty(w: usize, h: usize) -> Field {
        let width = w;
        let height = h;
        let values = (0..(width * height))
            .into_iter()
            .map(|_| 0.)
            .collect::<Vec<f32>>();
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
            .flat_map(|&e| e.to_le_bytes().to_vec() )
            .collect()
    }
}

// fn rules(slice: [CellType; 4]) -> [CellType; 4] {
//     use CellType::*;
//     match slice {
//         [Sand, Empty,
//          Empty, Empty] => [Empty, Empty,
//                            Sand, Empty],
//         [Sand, Sand,
//          Sand, Empty] => [Sand, Empty,
//                           Sand, Sand],
//         [Sand, Sand,
//          Empty, Empty] => [Empty, Empty,
//                            Sand, Sand],
//         [Empty, Sand,
//          Sand, Empty] => [Empty, Empty,
//                           Sand, Sand],
//         [Empty, Sand,
//          Empty, Empty] => [Empty, Empty,
//                            Empty, Sand],
//         [Sand, Sand,
//          Empty, Sand] => [Empty, Sand,
//                           Sand, Sand],
//         [Sand, Empty,
//          Empty, Sand] => [Empty, Empty,
//                           Sand, Sand],
//         [Sand, Empty,
//          Sand, Empty] => [Empty, Empty,
//                           Sand, Sand],
//         [Empty, Sand,
//          Empty, Sand] => [Empty, Empty,
//                           Sand, Sand],
//         otherwise => otherwise
//     }
// }

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
