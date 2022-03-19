use field::CellType;
use gl::attributes::AttributeVector2;
use gl::mesh::Mesh;
use gl::texture::ColorFormat;
use gl::texture::ColorFramebuffer;
use gl::texture::EmptyFramebuffer;
use gl::texture::Framebuffer;
use gl::texture::InternalFormat;
use gl::texture::TextureSpec;
use gl::texture::UploadedTexture;
use gl::texture::Viewport;
use gl::Ctx;
use gl::Pipeline;
use gl::Program;
use gl::GL;
use wasm_bindgen::prelude::*;

use std::collections::HashMap;

use glsmrs as gl;

use crate::field::texture;
use crate::field::PATTERNS;
use crate::field::RULES;

mod field;

pub fn make_quad() -> ([[f32; 2]; 4], [[f32; 2]; 4], [u16; 6]) {
    let vertices: [[f32; 2]; 4] = [[-1.0, -1.0], [1.0, -1.0], [1.0, 1.0], [-1.0, 1.0]];
    let uvs: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

    (vertices, uvs, indices)
}

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn display_shader(ctx: &Ctx) -> Result<gl::Program, JsValue> {
    gl::Program::new(
        ctx,
        include_str!("../shaders/dummy.vert"),
        include_str!("../shaders/display.frag"),
    )
    .map_err(JsValue::from)
}

pub fn copy_shader(ctx: &Ctx) -> Result<gl::Program, JsValue> {
    gl::Program::new(
        ctx,
        include_str!("../shaders/dummy.vert"),
        include_str!("../shaders/copy.frag"),
    )
    .map_err(JsValue::from)
}

pub fn update_shader(ctx: &Ctx) -> Result<gl::Program, JsValue> {
    gl::Program::new(
        ctx,
        include_str!("../shaders/dummy.vert"),
        include_str!("../shaders/compute.frag"),
    )
    .map_err(JsValue::from)
}

pub fn initial_state(ctx: &Ctx) -> Result<Mesh, String> {
    let (vertices, uvs, indices) = make_quad();

    Mesh::new(ctx, &indices)?
        .with_attribute::<AttributeVector2>("vert_position", &vertices)?
        .with_attribute::<AttributeVector2>("vert_uv", &uvs)
}

#[wasm_bindgen]
pub struct BrushStroke {
    x: f32,
    y: f32,
    color: CellType,
    radius: f32,
}

impl BrushStroke {
    pub fn new(x: f32, y: f32, color: CellType, radius: f32) -> Self {
        Self {
            x,
            y,
            color,
            radius,
        }
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    pub fn change_color(&mut self, color: CellType) {
        self.color = color;
    }

    pub fn change_radius(&mut self, radius: f32) {
        self.radius = radius
    }
}

impl Default for BrushStroke {
    fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            color: CellType::Empty,
            radius: 0.,
        }
    }
}

#[wasm_bindgen]
pub struct Render {
    pipeline: Pipeline,
    mesh: Mesh,
    brush: BrushStroke,
    display_shader: Program,
    update_shader: Program,
    copy_shader: Program,
    display_fb: EmptyFramebuffer,
    state_fb: ColorFramebuffer,
    temp_fb: ColorFramebuffer,
    patterns_texture: UploadedTexture,
    rules_texture: UploadedTexture,
    dimensions: [f32; 2],
}

#[wasm_bindgen]
impl Render {
    pub fn new(canvas_name: &str, w: u32, h: u32) -> Result<Render, JsValue> {
        console_error_panic_hook::set_once();

        let canvas = gl::util::get_canvas(canvas_name)
            .ok_or(format!("unable to find canvas {}", canvas_name))?;
        let ctx = Ctx::new(gl::util::get_ctx_from_canvas(&canvas, "webgl")?)?;

        let empty_bytes = field::Field::new(w as usize, h as usize, |_| CellType::Empty);

        let mesh = initial_state(&ctx)?;

        let display_shader = display_shader(&ctx)?;
        let copy_shader = copy_shader(&ctx)?;
        let update_shader = update_shader(&ctx)?;

        let display_fb =
            EmptyFramebuffer::new(&ctx, Viewport::new(canvas.width(), canvas.height()));

        let vp = Viewport::new(w, h);

        let patterns_texture = texture(&ctx, PATTERNS)?;
        let rules_texture = texture(&ctx, RULES)?;

        let texture_spec = TextureSpec::pixel(ColorFormat(GL::RGBA), [w, h]);
        let state_texture = texture_spec.upload(&ctx, InternalFormat(GL::UNSIGNED_BYTE), None)?;
        let state_fb = EmptyFramebuffer::new(&ctx, vp).with_color_slot(state_texture)?;

        let temp_fb = texture_spec.upload_u8(&ctx, &empty_bytes.bytes())?;
        let temp_fb = EmptyFramebuffer::new(&ctx, vp).with_color_slot(temp_fb)?;

        let pipeline = Pipeline::new(&ctx);

        Ok(Self {
            pipeline,
            mesh,
            brush: BrushStroke::default(),
            display_shader,
            copy_shader,
            update_shader,
            display_fb,
            state_fb,
            temp_fb,
            patterns_texture,
            rules_texture,
            dimensions: [w as f32, h as f32],
        })
    }

    pub fn brush_move_to(&mut self, x: f32, y: f32) {
        self.brush.move_to(x, y)
    }

    pub fn brush_change_radius(&mut self, radius: f32) {
        self.brush.change_radius(radius)
    }

    pub fn brush_change_color(&mut self, color: CellType) {
        self.brush.change_color(color);
    }

    pub fn frame(&mut self, time_step: f32) -> Result<(), String> {
        let uniforms = vec![
            ("num_rules", gl::UniformData::Scalar(RULES.len() as f32)),
            (
                "patterns",
                gl::UniformData::Texture(&mut self.patterns_texture),
            ),
            ("rules", gl::UniformData::Texture(&mut self.rules_texture)),
            (
                "position",
                gl::UniformData::Vector2([self.brush.x, self.brush.y]),
            ),
            ("color", gl::UniformData::Scalar(self.brush.color.into())),
            ("radius", gl::UniformData::Scalar(self.brush.radius)),
            ("field", gl::UniformData::Texture(self.temp_fb.color_slot())),
            ("field_size", gl::UniformData::Vector2(self.dimensions)),
            ("time_step", gl::UniformData::Scalar(time_step)),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        self.pipeline.shade(
            &self.update_shader,
            uniforms,
            vec![&mut self.mesh],
            &mut self.state_fb,
        )?;

        let copy_uniforms = vec![(
            "field",
            gl::UniformData::Texture(self.state_fb.color_slot()),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();

        self.pipeline.shade(
            &self.copy_shader,
            copy_uniforms,
            vec![&mut self.mesh],
            &mut self.temp_fb,
        )?;

        let display_uniforms = vec![(
            "field",
            gl::UniformData::Texture(self.state_fb.color_slot()),
        )]
        .into_iter()
        .collect::<HashMap<_, _>>();

        self.pipeline.shade(
            &self.display_shader,
            display_uniforms,
            vec![&mut self.mesh],
            &mut self.display_fb,
        )?;

        Ok(())
    }
}
