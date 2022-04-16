use glsmrs::{texture::{UploadedTexture, ColorFormat, TextureSpec}, Ctx, GL};
use wasm_bindgen::prelude::wasm_bindgen;

use hex_color::HexColor;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[wasm_bindgen]
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, EnumIter)]
pub enum CellType {
    Empty,
    // Water,
    Sand,
    Wall,
}

#[wasm_bindgen]
pub fn color_hex(cell: CellType) -> String {
    let color = cell.color();
    HexColor::new(
        (color[0] * 255.) as u8,
        (color[1] * 255.) as u8,
        (color[2] * 255.) as u8,
    )
    .to_string()
}

impl CellType {
    pub fn pack_rgba(&self) -> [f32; 4] {
        let idx = Self::iter().take_while(|c| c != self).count() as f32;
        [(idx + 0.5) / Self::iter().len() as f32, 0., 0., 0.]
    }

    fn color(self) -> [f32; 4] {
        use CellType::*;
        match self {
            Empty => [0., 0., 0., 1.],
            Sand => [168. / 255., 134. / 255., 42. / 255., 1.],
            // Water => [103. / 255., 133. / 255., 193. / 255., 1.],
            Wall => [148. / 255., 148. / 255., 148. / 255., 1.],
        }
    }

    pub fn color_texture_bytes(ctx: &Ctx) -> Result<UploadedTexture, String> {
        let tex = TextureSpec::pixel(ColorFormat(GL::RGBA), [Self::iter().len() as u32, 1]);
        let bytes = Self::iter().map(Self::color).collect::<Vec<[f32; 4]>>();
        tex.upload_rgba(ctx, &bytes)
    }
}
