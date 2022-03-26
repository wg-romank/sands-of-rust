use std::collections::HashMap;

use glsmrs::texture::{ColorFormat, TextureSpec, UploadedTexture};
use glsmrs::{Ctx, GL};

use crate::cell::CellType;
use crate::cell::CellType::*;

pub struct Rules {
    rules: HashMap<[CellType; 4], [CellType; 4]>,
}

impl Rules {
    pub fn new() -> Self {
        Self {
            rules: vec![
                ([Sand, Empty, Empty, Empty], [Empty, Empty, Sand, Empty]),
                ([Sand, Sand, Sand, Empty], [Sand, Empty, Sand, Sand]),
                ([Sand, Sand, Empty, Empty], [Empty, Empty, Sand, Sand]),
                ([Empty, Sand, Sand, Empty], [Empty, Empty, Sand, Sand]),
                ([Empty, Sand, Empty, Empty], [Empty, Empty, Empty, Sand]),
                ([Sand, Sand, Empty, Sand], [Empty, Sand, Sand, Sand]),
                ([Sand, Empty, Empty, Sand], [Empty, Empty, Sand, Sand]),
                ([Sand, Empty, Sand, Empty], [Empty, Empty, Sand, Sand]),
                ([Empty, Sand, Empty, Sand], [Empty, Empty, Sand, Sand]),
                ([Sand, Wall, Empty, Empty], [Empty, Wall, Sand, Empty]),
                ([Wall, Sand, Empty, Empty], [Wall, Empty, Empty, Sand]),
                ([Sand, Wall, Empty, Wall], [Empty, Wall, Sand, Wall]),
                ([Wall, Sand, Wall, Empty], [Wall, Empty, Wall, Sand]),
                ([Sand, Empty, Empty, Wall], [Empty, Empty, Sand, Wall]),
                ([Empty, Sand, Wall, Empty], [Empty, Empty, Wall, Sand]),
                ([Sand, Empty, Wall, Empty], [Empty, Empty, Wall, Sand]),
                ([Empty, Sand, Empty, Wall], [Empty, Empty, Sand, Wall]),
            ]
            .into_iter()
            .collect::<HashMap<_, _>>(),
        }
    }

    pub fn num_rules(&self) -> f32 {
        self.rules.len() as f32
    }

    pub fn to_textures(&self, ctx: &Ctx) -> Result<(UploadedTexture, UploadedTexture), String> {
        let patterns = self.rules.keys().map(Clone::clone).collect::<Vec<_>>();
        let rules = self.rules.values().map(Clone::clone).collect::<Vec<_>>();

        let patterns_tex = texture(ctx, patterns)?;
        let rules_tex = texture(ctx, rules)?;

        Ok((patterns_tex, rules_tex))
    }

    #[cfg(test)]
    pub fn rules(&self, slice: [CellType; 4]) -> [CellType; 4] {
        self.rules.get(&slice).unwrap_or(&slice).clone()
    }
}

pub fn texture(ctx: &Ctx, arr: Vec<[CellType; 4]>) -> Result<UploadedTexture, String> {
    let useful_size = 2 * arr.len() as u32;
    // todo: compute atuomatically
    let next_po2 = (0..)
        .map(|n| 2u32.pow(n))
        .find(|&n| n > useful_size)
        .ok_or(format!("failed to find suitable size for texture, array too big {}?", useful_size))?;

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
