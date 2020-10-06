use wasm_bindgen::prelude::*;

use wasm_bindgen::JsCast;
use web_sys;
use std::collections::HashMap;

use glsmrs as gl;

mod field;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn get_canvas() -> Option<web_sys::HtmlCanvasElement> {
    let document = web_sys::window()?.document()?;
    let canvas = document.get_element_by_id("sands-of-rust-canvas")?;

    canvas.dyn_into::<web_sys::HtmlCanvasElement>().ok()
}

fn get_ctx<T : JsCast>(ctx_name: &str) -> Result<T, JsValue> {
    let ctx = get_canvas()
        .ok_or(JsValue::from_str("Failed to get canvas"))?
        .get_context(ctx_name)?
        .ok_or(JsValue::from_str("Failed getting ctx"))?;

    ctx.dyn_into::<T>()
        .map_err(|e| JsValue::from(e))
}

pub fn make_quad() -> ([f32; 8], [f32; 8], [u16; 6]) {
    let vertices: [f32; 8] = [
        -1.0, -1.0,
        1.0, -1.0,
        1.0, 1.0,
        -1.0, 1.0
    ];
    let uvs: [f32; 8] = [
        0.0, 0.0,
        1.0, 0.0,
        1.0, 1.0,
        0.0, 1.0
    ];
    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

    (vertices, uvs, indices)
}


#[wasm_bindgen]
pub fn display_shader() -> Result<gl::Program, JsValue> {
    let ctx = get_ctx("webgl")?;

    gl::Program::new(
        &ctx,
        include_str!("../shaders/dummy.vert"),
        include_str!("../shaders/display.frag"),
        vec![
            gl::UniformDescription::new("field", gl::UniformType::Sampler2D),
        ],
        vec![
            gl::AttributeDescription::new("vert_position", gl::AttributeType::Vector2),
            gl::AttributeDescription::new("vert_uv", gl::AttributeType::Vector2),
        ],
    ).map_err(|e| JsValue::from(e)) 
}

#[wasm_bindgen]
pub fn copy_shader() -> Result<gl::Program, JsValue> {
    let ctx = get_ctx("webgl")?;

    gl::Program::new(
        &ctx,
        include_str!("../shaders/dummy.vert"),
        include_str!("../shaders/copy.frag"),
        vec![
            gl::UniformDescription::new("field", gl::UniformType::Sampler2D),
        ],
        vec![
            gl::AttributeDescription::new("vert_position", gl::AttributeType::Vector2),
            gl::AttributeDescription::new("vert_uv", gl::AttributeType::Vector2),
        ],
    ).map_err(|e| JsValue::from(e))
}

#[wasm_bindgen]
pub fn update_shader() -> Result<gl::Program, JsValue> {
    let ctx = get_ctx("webgl")?;

    gl::Program::new(
        &ctx,
        include_str!("../shaders/dummy.vert"),
        include_str!("../shaders/compute.frag"),
        vec![
            gl::UniformDescription::new("field", gl::UniformType::Sampler2D),
            // gl::UniformDescription::new("external_force", gl::UniformType::Sampler2D),
            gl::UniformDescription::new("field_size", gl::UniformType::Vector2),
            gl::UniformDescription::new("time_step", gl::UniformType::Float),
        ],
        vec![
            gl::AttributeDescription::new("vert_position", gl::AttributeType::Vector2),
            gl::AttributeDescription::new("vert_uv", gl::AttributeType::Vector2),
        ],
    ).map_err(|e| JsValue::from(e))
}

#[wasm_bindgen]
pub fn initial_state(
    force_field: &field::Field,
    w: u32,
    h: u32,
) -> Result<gl::GlState, JsValue> {
    let canvas = get_canvas().ok_or(JsValue::from_str("Failed to get canvas"))?;

    let context = get_ctx("webgl")?;

    let (vertices, uvs, indices) = make_quad();

    let mut state = gl::GlState::new(&context, gl::Viewport {w: canvas.width(), h: canvas.height()});

    // attribute arrays need to be packed with little-endinan bytes
    // while texture data is big-endian X_X
    let packf32 = |v: &[f32]| { v.iter().flat_map(|el| el.to_le_bytes().to_vec()).collect::<Vec<u8>>() };
    let packu16 = |v: &[u16]| { v.iter().flat_map(|el| el.to_le_bytes().to_vec()).collect::<Vec<u8>>() };

    let empty_bytes = field::Field::new_empty(w as usize, h as usize, field::CellType::Empty);

    let state_bytes = field::Field::new(w as usize, h as usize);

    state
        .vertex_buffer("vert_position", packf32(&vertices).as_slice())?
        .vertex_buffer("vert_uv", packf32(&uvs).as_slice())?
        .element_buffer(packu16(&indices).as_slice())?
        .texture("state", Some(&state_bytes.bytes().as_slice()), w, h)?
        .texture("display", Some(&state_bytes.bytes().as_slice()), w, h)?
        .texture("force_field", Some(&force_field.bytes().as_slice()), w, h)?;

    Ok(state)
}

#[wasm_bindgen]
pub fn animation_frame(
    display_shader: &gl::Program,
    update_shader: &gl::Program,
    copy_shader: &gl::Program,
    force_field: &mut field::Field,
    state: &mut gl::GlState,
    time_step: f32,
) -> Result<(), JsValue> {
    let uniforms = vec![
        ("field", gl::UniformData::Texture("display")),
        ("external_force", gl::UniformData::Texture("force_field")),
        ("field_size", gl::UniformData::Vector2([force_field.width as f32, force_field.height as f32])),
        ("time_step", gl::UniformData::Scalar(time_step)),
    ].into_iter().collect::<HashMap<_, _>>();

    let copy_uniforms = vec![
        ("field", gl::UniformData::Texture("state")),
    ].into_iter().collect::<HashMap<_, _>>();

    let w = force_field.width as u32;
    let h = force_field.height as u32;

    // force_field.step(time_step as u32);

    state
        // .texture("force_field", Some(&force_field.bytes().as_slice()), w, h)?
        // .texture("state", Some(&force_field.bytes().as_slice()), w, h)?
        .run_mut(update_shader, &uniforms, "state")?
        .run_mut(copy_shader, &copy_uniforms, "display")?
        .run(display_shader, &copy_uniforms)?;

    Ok(())
}