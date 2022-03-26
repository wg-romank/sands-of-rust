use glsmrs::texture::{ColorFormat, TextureSpec, UploadedTexture};
use glsmrs::{Ctx, GL};

use crate::cell::CellType;
use crate::cell::CellType::*;

pub struct Rules {
    rules: Vec<([CellType; 4], [CellType; 4])>,
    useful_size: u32,
    next_po2: u32,
}

impl Rules {
    pub fn with_symmetry(pat: Vec<([CellType; 4], [CellType; 4])>) -> Vec<([CellType; 4], [CellType; 4])> {
        pat.into_iter().flat_map(|(p, r)| vec![
            (p, r),
            // 0 1
            // 2 3
            ([p[1], p[0], p[3], p[2]], [r[1], r[0], r[3], r[2]])
        ]).collect()
    }

    pub fn new() -> Self {
        let rules = Self::with_symmetry(vec![
            // * ~
            // ~ ~
            ([Sand, Empty, Empty, Empty], [Empty, Empty, Sand, Empty]),
            // * *
            // * ~
            ([Sand, Sand, Sand, Empty], [Sand, Empty, Sand, Sand]),
            // * *
            // ~ ~
            ([Sand, Sand, Empty, Empty], [Empty, Empty, Sand, Sand]),
            // * ~
            // ~ *
            ([Sand, Empty, Empty, Sand], [Empty, Empty, Sand, Sand]),
            // * ~
            // * ~
            ([Sand, Empty, Sand, Empty], [Empty, Empty, Sand, Sand]),
            // * W
            // ~ ~
            ([Sand, Wall, Empty, Empty], [Empty, Wall, Sand, Empty]),
            // * W
            // ~ W
            ([Sand, Wall, Empty, Wall], [Empty, Wall, Sand, Wall]),
            // * ~
            // ~ W
            ([Sand, Empty, Empty, Wall], [Empty, Empty, Sand, Wall]),
            // * ~
            // W ~
            ([Sand, Empty, Wall, Empty], [Empty, Empty, Wall, Sand]),
        ]);

        let useful_size = rules.len() as u32 * 2;
        let next_po2 = (0..)
            .map(|n| 2u32.pow(n))
            .find(|&n| n > useful_size)
            .unwrap();

        Self { rules, next_po2, useful_size }
    }

    pub fn num_rules(&self) -> f32 {
        self.rules.len() as f32
    }

    pub fn texture_len(&self) -> f32 {
        self.next_po2 as f32
    }

    pub fn to_textures(&self, ctx: &Ctx) -> Result<(UploadedTexture, UploadedTexture), String> {
        let patterns = self.rules.iter().map(|(p, _)| p.clone()).collect::<Vec<_>>();
        let rules = self.rules.iter().map(|(_, r)| r.clone()).collect::<Vec<_>>();

        let patterns_tex = texture(ctx, patterns, self.useful_size, self.next_po2)?;
        let rules_tex = texture(ctx, rules, self.useful_size, self.next_po2)?;

        Ok((patterns_tex, rules_tex))
    }

    #[cfg(test)]
    pub fn rules(&self, slice: [CellType; 4]) -> [CellType; 4] {
        self.rules.iter().find(|(p, _)| p == &slice).map(|(_, r)| r).unwrap_or(&slice).clone()
    }
}

pub fn texture(ctx: &Ctx, arr: Vec<[CellType; 4]>, useful_size: u32, next_po2: u32) -> Result<UploadedTexture, String> {
    let padding = (0..(next_po2 - useful_size))
        .map(|_| [0., 0., 0., 0.])
        .collect::<Vec<[f32; 4]>>();
    let spec = TextureSpec::pixel(ColorFormat(GL::RGBA), [next_po2, 2]);

    let mut line1 = arr
        .iter()
        .flat_map(|ar| {
            [ar[0], ar[1]]
                .iter()
                .map(CellType::pack_rgba)
                .collect::<Vec<[f32; 4]>>()
        })
        .collect::<Vec<[f32; 4]>>();
    line1.extend(&padding);

    let line2 = arr
        .iter()
        .flat_map(|ar| {
            [ar[2], ar[3]]
                .iter()
                .map(CellType::pack_rgba)
                .collect::<Vec<[f32; 4]>>()
        })
        .collect::<Vec<[f32; 4]>>();

    line1.extend(line2);
    line1.extend(&padding);

    spec.upload_rgba(ctx, &line1)
}
