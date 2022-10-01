extern crate js_sys;
extern crate mat4;
extern crate wasm_bindgen;
extern crate web_sys;

use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext};

#[allow(dead_code)]
mod utils;
use utils::{compile_shader, link_program, set_panic_hook};

#[derive(Debug, Clone)]
struct Buffers(WebGlBuffer, WebGlBuffer);

#[wasm_bindgen]
extern "C" {
    pub fn alert(s: &str);
}

#[wasm_bindgen]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExampleKind {
    Sin = 0,
    Custom = 1,
}

#[wasm_bindgen]
pub struct LChart {}

#[derive(Debug, Clone)]
struct ProgramInfo {
    shader_program: WebGlProgram,
    vertex_position_ptr: u32,
}

#[wasm_bindgen]
impl LChart {
    pub fn new() -> Self {
        LChart {}
    }

    pub fn test(
        &self,
        gl: &WebGlRenderingContext,
        kind: ExampleKind,
        point_count: usize,
        from_x: f32,
        to_x: f32,
        from_y: f32,
        to_y: f32,
    ) -> Result<(), JsValue> {
        let delta = (to_x - from_x) / ((point_count - 1) as f32);
        let mut x = from_x;
        let mut points = Vec::<f32>::with_capacity(2 * point_count);
        while (x < to_x) {
            points.push(x);
            points.push(x.sin());
            x = x + delta;
        }
        points.push(to_x);
        points.push(to_x.sin());
        alert(&format!("Delta is {}, count is {}", delta, points.len()));
        Ok(())
    }

    pub fn draw(&self, gl: &WebGlRenderingContext) -> Result<(), JsValue> {
        // gl.viewport(0, 0, 400, 300);

        // Vertex shader program
        let vsSource = r#"
            attribute vec3 aVertexPosition;
            void main(void) {
                gl_Position = vec4(aVertexPosition, 1.0);
            }
        "#;

        // Fragment shader program
        let fsSource = r#"
            void main(void) {
                gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
            }
        "#;

        // Initialize a shader program; this is where all the lighting
        // for the vertices and so forth is established.
        let shader_program = initShaderProgram(gl, vsSource, fsSource)?;

        // Collect all the info needed to use the shader program.
        // Look up which attributes our shader program is using
        // for aVertexPosition, aVevrtexColor and also
        // look up uniform locations.
        let program_info = {
            let vertex_position_ptr =
                gl.get_attrib_location(&shader_program, "aVertexPosition") as u32;
            ProgramInfo {
                shader_program,
                vertex_position_ptr,
            }
        };

        // Here's where we call the routine that builds all the
        // objects we'll be drawing.
        let buffers: Buffers = init_buffers(gl)?;

        gl.clear_color(1.0, 0.0, 0.0, 1.0);

        // Tell WebGL to use our program when drawing
        gl.use_program(Some(&program_info.shader_program));
        gl.enable_vertex_attrib_array(program_info.vertex_position_ptr);

        // draw
        let canvas: web_sys::HtmlCanvasElement = gl
            .canvas()
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        gl.vertex_attrib_pointer_with_i32(
            program_info.vertex_position_ptr, // shaderProgram.vertexPositionAttribute,
            3,                                // vertexBuffer.itemSize,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        // отрисовка примитивов - линий
        gl.draw_elements_with_i32(
            WebGlRenderingContext::LINE_STRIP,
            6, // indexBuffer.numberOfItems,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );
        // alert(&format!("Hello"));
        Ok(())
    }
}

#[allow(non_snake_case)]
fn initShaderProgram(
    gl: &WebGlRenderingContext,
    vsSource: &str,
    fsSource: &str,
) -> Result<WebGlProgram, String> {
    let v_shader = compile_shader(gl, WebGlRenderingContext::VERTEX_SHADER, vsSource);
    let f_shader = compile_shader(gl, WebGlRenderingContext::FRAGMENT_SHADER, fsSource);

    link_program(gl, &v_shader?, &f_shader?)
}

#[allow(non_snake_case)]
fn init_buffers(gl: &WebGlRenderingContext) -> Result<Buffers, JsValue> {
    // Create a buffer for the vertex positions
    let position_buffer = gl
        .create_buffer()
        .ok_or("failed to create positionBuffer buffer")?;

    // Select the positionBuffer as the one to apply buffer
    // operations to from here out.
    gl.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&position_buffer));

    // Now create an array of positions
    let positions: [f32; 12] = [
        -0.5, -0.5, 0.0, -0.5, 0.5, 0.0, 0.5, 0.5, 0.0, 0.5, -0.5, 0.0,
    ];
    let position_array = float_32_array!(positions);
    // Now pass the list of positions into WebGL to build the
    // shape. We do this by creating a Float32Array from the
    // Rust array, then use it to fill the current buffer.
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ARRAY_BUFFER,
        &position_array,
        WebGlRenderingContext::STATIC_DRAW,
    );

    // Build the element array buffer; this specifies the indices
    // into the vertex arrays for each face's vertices.
    let index_buffer = gl
        .create_buffer()
        .ok_or("failed to create indexBuffer buffer")?;
    gl.bind_buffer(
        WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
        Some(&index_buffer),
    );

    // This array defines each face as two triangles, using the
    // indices into the vertex array to specify each triangle's
    // position.
    let indices: [u16; 6] = [0, 1, 2, 0, 3, 2];
    let index_array = uint_16_array!(indices);
    gl.buffer_data_with_array_buffer_view(
        WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
        &index_array,
        WebGlRenderingContext::STATIC_DRAW,
    );
    Ok(Buffers(position_buffer, index_buffer))
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}
