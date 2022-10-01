extern crate js_sys;
extern crate mat4;
extern crate wasm_bindgen;
extern crate web_sys;

use js_sys::WebAssembly;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlBuffer, WebGlProgram, WebGlRenderingContext, WebGlUniformLocation};

#[allow(dead_code)]
mod utils;
use utils::{compile_shader, link_program};

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
pub struct LChart {
    gl: WebGlRenderingContext,
    program: ProgramInfo,
    indices: Vec<u16>,
    vertexes: Vec<f32>,
    matrix: Vec<f32>,
}

#[derive(Debug, Clone)]
struct ProgramInfo {
    shader_program: WebGlProgram,
    vertex_position_ptr: u32,
    u_matrix: Result<WebGlUniformLocation, String>,
    buffer_vertex: WebGlBuffer,
    buffer_indices: WebGlBuffer,
}

#[wasm_bindgen]
impl LChart {
    pub fn new(gl: &WebGlRenderingContext) -> Result<LChart, JsValue> {
        // Vertex shader program
        let vs_source = r#"
            attribute vec2 a_position;

            uniform mat3 u_matrix;
            void main(void) {
                gl_Position = vec4(u_matrix * vec3(a_position, 1.0), 1.0);
            }
        "#;

        // Fragment shader program
        let fs_source = r#"
            void main(void) {
                gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
            }
        "#;

        // Initialize a shader program; this is where all the lighting
        // for the vertices and so forth is established.
        let shader_program = initShaderProgram(gl, vs_source, fs_source)?;

        let program_info = {
            let vertex_position_ptr = gl.get_attrib_location(&shader_program, "a_position") as u32;
            let Buffers(buffer_vertex, buffer_indexes) = create_buffers(gl)?;
            let u_matrix = gl
                .get_uniform_location(&shader_program, "u_matrix")
                .ok_or_else(|| String::from("cannot get u_matrix"));
            ProgramInfo {
                shader_program,
                vertex_position_ptr,
                buffer_vertex,
                buffer_indices: buffer_indexes,
                u_matrix,
            }
        };

        gl.clear_color(1.0, 0.0, 0.0, 1.0);

        let chart = LChart {
            gl: gl.clone(),
            program: program_info,
            indices: vec![],
            vertexes: vec![],
            matrix: vec![],
        };

        // chart.draw()?;

        Ok(chart)
    }

    fn draw(&self) -> Result<(), JsValue> {
        let gl = &self.gl;
        let program = self.program.clone();
        // Tell WebGL to use our program when drawing

        gl.use_program(Some(&program.shader_program));
        gl.enable_vertex_attrib_array(program.vertex_position_ptr);

        self.init_buffers(&self.indices, &self.vertexes)?;

        // draw
        let canvas: web_sys::HtmlCanvasElement = gl
            .canvas()
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()?;
        gl.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        gl.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);
        gl.vertex_attrib_pointer_with_i32(
            program.vertex_position_ptr, // shaderProgram.vertexPositionAttribute,
            2,                           // vertexBuffer.itemSize,
            WebGlRenderingContext::FLOAT,
            false,
            0,
            0,
        );
        gl.uniform_matrix3fv_with_f32_array(Some(&program.u_matrix?), false, &self.matrix);
        // отрисовка примитивов - линий
        gl.draw_elements_with_i32(
            WebGlRenderingContext::LINE_STRIP,
            self.indices.len() as i32, // indexBuffer.numberOfItems,
            WebGlRenderingContext::UNSIGNED_SHORT,
            0,
        );
        Ok(())
    }

    fn init_buffers(&self, indices: &[u16], positions: &[f32]) -> Result<(), JsValue> {
        let gl = &self.gl;
        let program = self.program.clone();
        // Select the positionBuffer as the one to apply buffer
        // operations to from here out.
        gl.bind_buffer(
            WebGlRenderingContext::ARRAY_BUFFER,
            Some(&program.buffer_vertex),
        );

        let position_array = float_32_array!(positions);
        // Now pass the list of positions into WebGL to build the
        // shape. We do this by creating a Float32Array from the
        // Rust array, then use it to fill the current buffer.
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ARRAY_BUFFER,
            &position_array,
            WebGlRenderingContext::STATIC_DRAW,
        );

        gl.bind_buffer(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&program.buffer_indices),
        );

        // This array defines each face as two triangles, using the
        // indices into the vertex array to specify each triangle's
        // position.
        let index_array = uint_16_array!(indices);
        gl.buffer_data_with_array_buffer_view(
            WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
            &index_array,
            WebGlRenderingContext::STATIC_DRAW,
        );
        Ok(())
    }

    pub fn test(
        &mut self,
        _kind: ExampleKind,
        point_count: usize,
        from_x: f32,
        to_x: f32,
        from_y: f32,
        to_y: f32,
    ) -> Result<(), JsValue> {
        let delta = (to_x - from_x) / ((point_count - 1) as f32);
        let mut x = from_x;
        let mut points = Vec::<f32>::with_capacity(2 * point_count);
        while x < to_x {
            points.push(x);
            points.push(x.sin());
            x += delta;
        }
        points.push(to_x);
        points.push(to_x.sin());
        self.matrix = vec![
            2.0 / (to_x - from_x),
            0.0,
            0.0,
            0.0,
            -2.0 / (to_y - from_y),
            0.0,
            -1.0 - from_x,
            -1.0 - from_y,
            1.0,
        ];
        self.indices = points.iter().enumerate().map(|(i, _)| i as u16).collect();
        self.vertexes = points;
        self.draw()?;
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
fn create_buffers(gl: &WebGlRenderingContext) -> Result<Buffers, JsValue> {
    // Create a buffer for the vertex positions
    let position_buffer = gl
        .create_buffer()
        .ok_or("failed to create positionBuffer buffer")?;

    // Build the element array buffer; this specifies the indices
    // into the vertex arrays for each face's vertices.
    let index_buffer = gl
        .create_buffer()
        .ok_or("failed to create indexBuffer buffer")?;

    Ok(Buffers(position_buffer, index_buffer))
}

#[wasm_bindgen]
pub fn greet(name: &str) {
    alert(&format!("Hello, {}!", name));
}
