mod utils;

use std::cell::RefCell;
use std::rc::Rc;
use std::f64;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader, WebGlBuffer};
use js_sys;
use gl_matrix::mat4;

use crate::utils as rusty_utils;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}


#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    rusty_utils::set_panic_hook();

    let canvas = document().get_element_by_id("viewer").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    let vert_shader = compile_shader(
        &context,
        WebGlRenderingContext::VERTEX_SHADER,
        r#"
        attribute vec4 aVertexPosition;
        attribute vec4 aVertexColor;

        uniform mat4 uModelViewMatrix;
        uniform mat4 uProjectionMatrix;

        varying lowp vec4 vColor;

        void main(void) {
            gl_Position = uProjectionMatrix * uModelViewMatrix * aVertexPosition;
            vColor = aVertexColor;
        }
    "#,
    )?;
    let frag_shader = compile_shader(
        &context,
        WebGlRenderingContext::FRAGMENT_SHADER,
        r#"
        varying lowp vec4 vColor;

        void main(void) {
            gl_FragColor = vColor;
        }
    "#,
    )?;
    let program = init_shader_program(&context, &vert_shader, &frag_shader)?;

    let vertex_position = context.get_attrib_location(&program, "aVertexPosition");
    let vertex_color = context.get_attrib_location(&program, "aVertexColor");
    let projection_matrix = context.get_uniform_location(&program, "uProjectionMatrix").unwrap();
    let model_view_matrix = context.get_uniform_location(&program, "uModelViewMatrix").unwrap();

    let buffers = init_buffers(&context).unwrap();

    let mut cube_rotation = 0 as f32;

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        context.clear_color(0.0, 0.0, 0.0, 1.0);            // Clear to black, fully opaque
        context.clear_depth(1.0);                           // Clear everything
        context.enable(WebGlRenderingContext::DEPTH_TEST);  // Enable depth testing
        context.depth_func(WebGlRenderingContext::LEQUAL);  // Near things obscure far things

        context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT | WebGlRenderingContext::DEPTH_BUFFER_BIT);

        let field_of_view = 45.0 * f64::consts::PI / 180.0; // in radians
        let aspect = canvas.client_width() / canvas.client_height();
        let z_near = 0.1;
        let z_far = 100.0;
        let mut internal_projection_matrix = mat4::create();

        mat4::perspective(
            &mut internal_projection_matrix,
            field_of_view as f32,
            aspect as f32,
            z_near,
            Some(z_far)
        );

        let mut internal_model_view_matrix = mat4::create();
        let mut internal_model_view_matrix_2 = internal_model_view_matrix.clone();

        mat4::translate(
            &mut internal_model_view_matrix,
            &mut internal_model_view_matrix_2,
            &[-0.0, 0.0, -6.0]
        );

        // We need to copy after _every_ translation/rotation -_-
        internal_model_view_matrix_2 = internal_model_view_matrix.clone();

        mat4::rotate(
            &mut internal_model_view_matrix,
            &mut internal_model_view_matrix_2,
            cube_rotation,
            &[0.0, 0.0, 1.0]
        );

        // We need to copy after _every_ translation/rotation -_-
        internal_model_view_matrix_2 = internal_model_view_matrix.clone();

        mat4::rotate(
            &mut internal_model_view_matrix,
            &mut internal_model_view_matrix_2,
            cube_rotation * 0.7,
            &[0.0, 1.0, 0.0]
        );

        {
            let num_components = 3;
            let component_type = WebGlRenderingContext::FLOAT;
            let normalize = false;
            let stride = 0;
            let offset = 0.0;

            context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffers.position));
            context.vertex_attrib_pointer_with_f64(
                vertex_position as u32,
                num_components,
                component_type,
                normalize,
                stride,
                offset
            );

            context.enable_vertex_attrib_array(vertex_position as u32);
        }

        {
            let num_components = 4;
            let component_type = WebGlRenderingContext::FLOAT;
            let normalize = false;
            let stride = 0;
            let offset = 0.0;

            context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffers.color));
            context.vertex_attrib_pointer_with_f64(
                vertex_color as u32,
                num_components,
                component_type,
                normalize,
                stride,
                offset
            );

            context.enable_vertex_attrib_array(vertex_color as u32);
        }

        context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&buffers.indices));

        context.use_program(Some(&program));

        context.uniform_matrix4fv_with_f32_array(
            Some(&projection_matrix),
            false,
            &internal_projection_matrix
        );

        log(format!("projection_matrix: {:?}", internal_projection_matrix).as_str());

        context.uniform_matrix4fv_with_f32_array(
            Some(&model_view_matrix),
            false,
            &internal_model_view_matrix
        );

        log(format!("model_view_matrix: {:?}", internal_model_view_matrix).as_str());

        {
            let vertex_count = 36;
            let vertex_type = WebGlRenderingContext::UNSIGNED_SHORT;
            let offset = 0.0;
            context.draw_elements_with_f64(
                WebGlRenderingContext::TRIANGLES,
                vertex_count,
                vertex_type,
                offset
            );
        }

        cube_rotation += 0.006;

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

pub fn compile_shader(
    context: &WebGlRenderingContext,
    shader_type: u32,
    source: &str,
) -> Result<WebGlShader, String> {
    let shader = context
        .create_shader(shader_type)
        .ok_or_else(|| String::from("Unable to create shader object"))?;
    context.shader_source(&shader, source);
    context.compile_shader(&shader);

    if context
        .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(shader)
    } else {
        Err(context
            .get_shader_info_log(&shader)
            .unwrap_or_else(|| String::from("Unknown error creating shader")))
    }
}

pub fn init_shader_program(
    context: &WebGlRenderingContext,
    vert_shader: &WebGlShader,
    frag_shader: &WebGlShader,
) -> Result<WebGlProgram, String> {
    let program = context
        .create_program()
        .ok_or_else(|| String::from("Unable to create shader object"))?;

    context.attach_shader(&program, vert_shader);
    context.attach_shader(&program, frag_shader);
    context.link_program(&program);

    if context
        .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        Ok(program)
    } else {
        Err(context
            .get_program_info_log(&program)
            .unwrap_or_else(|| String::from("Unknown error creating program object")))
    }
}

fn init_buffers(context: &WebGlRenderingContext) -> Result<Buffers, String> {
    // Setup the positions buffer

    let position_buffer = context.create_buffer().ok_or("failed to create position buffer")?;
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));

    let positions: [f32; 72] = [
        // Front face
        -1.0, -1.0,  1.0,
        1.0, -1.0,  1.0,
        1.0,  1.0,  1.0,
        -1.0,  1.0,  1.0,

        // Back face
        -1.0, -1.0, -1.0,
        -1.0,  1.0, -1.0,
        1.0,  1.0, -1.0,
        1.0, -1.0, -1.0,

        // Top face
        -1.0,  1.0, -1.0,
        -1.0,  1.0,  1.0,
        1.0,  1.0,  1.0,
        1.0,  1.0, -1.0,

        // Bottom face
        -1.0, -1.0, -1.0,
        1.0, -1.0, -1.0,
        1.0, -1.0,  1.0,
        -1.0, -1.0,  1.0,

        // Right face
        1.0, -1.0, -1.0,
        1.0,  1.0, -1.0,
        1.0,  1.0,  1.0,
        1.0, -1.0,  1.0,

        // Left face
        -1.0, -1.0, -1.0,
        -1.0, -1.0,  1.0,
        -1.0,  1.0,  1.0,
        -1.0,  1.0, -1.0,
    ];

    // More info on why this is unsafe
    // https://docs.rs/js-sys/0.3.45/js_sys/struct.Float32Array.html#unsafety
    unsafe {
        let position_array = js_sys::Float32Array::view(&positions);

        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &position_array,
            WebGlRenderingContext::STATIC_DRAW
        )
    }

    // Setup the color buffer

    let colors: [f32; 96] = [
        1.0,  1.0,  1.0,  1.0,    // Front face: white
        1.0,  1.0,  1.0,  1.0,    // Front face: white
        1.0,  1.0,  1.0,  1.0,    // Front face: white
        1.0,  1.0,  1.0,  1.0,    // Front face: white
        1.0,  0.0,  0.0,  1.0,    // Back face: red
        1.0,  0.0,  0.0,  1.0,    // Back face: red
        1.0,  0.0,  0.0,  1.0,    // Back face: red
        1.0,  0.0,  0.0,  1.0,    // Back face: red
        0.0,  1.0,  0.0,  1.0,    // Top face: green
        0.0,  1.0,  0.0,  1.0,    // Top face: green
        0.0,  1.0,  0.0,  1.0,    // Top face: green
        0.0,  1.0,  0.0,  1.0,    // Top face: green
        0.0,  0.0,  1.0,  1.0,    // Bottom face: blue
        0.0,  0.0,  1.0,  1.0,    // Bottom face: blue
        0.0,  0.0,  1.0,  1.0,    // Bottom face: blue
        0.0,  0.0,  1.0,  1.0,    // Bottom face: blue
        1.0,  1.0,  0.0,  1.0,    // Right face: yellow
        1.0,  1.0,  0.0,  1.0,    // Right face: yellow
        1.0,  1.0,  0.0,  1.0,    // Right face: yellow
        1.0,  1.0,  0.0,  1.0,    // Right face: yellow
        1.0,  0.0,  1.0,  1.0,    // Left face: purple
        1.0,  0.0,  1.0,  1.0,    // Left face: purple
        1.0,  0.0,  1.0,  1.0,    // Left face: purple
        1.0,  0.0,  1.0,  1.0,    // Left face: purple
    ];


    let color_buffer = context.create_buffer().ok_or("failed to create color buffer")?;
    context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&color_buffer));

    // More info on why this is unsafe
    // https://docs.rs/js-sys/0.3.45/js_sys/struct.Float32Array.html#unsafety
    unsafe {
        let color_array = js_sys::Float32Array::view(&colors);

        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &color_array,
            WebGlRenderingContext::STATIC_DRAW
        )
    }

    let index_buffer = context.create_buffer().ok_or("failed to create index buffer")?;
    context.bind_buffer(WebGlRenderingContext::ELEMENT_ARRAY_BUFFER, Some(&index_buffer));

    let indices = [
        0,  1,  2,      0,  2,  3,    // front
        4,  5,  6,      4,  6,  7,    // back
        8,  9,  10,     8,  10, 11,   // top
        12, 13, 14,     12, 14, 15,   // bottom
        16, 17, 18,     16, 18, 19,   // right
        20, 21, 22,     20, 22, 23,   // left
    ];

    // More info on why this is unsafe
    // https://docs.rs/js-sys/0.3.45/js_sys/struct.Float32Array.html#unsafety
    unsafe {
        let index_array = js_sys::Uint16Array::view(&indices);

        context.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &index_array,
            WebGlRenderingContext::STATIC_DRAW
        )
    }

    Ok(Buffers {
        position: position_buffer,
        color: color_buffer,
        indices: index_buffer
    })
}

struct Buffers {
    position: WebGlBuffer,
    color: WebGlBuffer,
    indices: WebGlBuffer,
}
