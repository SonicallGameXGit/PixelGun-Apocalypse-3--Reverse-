//! Thin OpenGL 3.3 helpers over `glow`.
//!
//! `unsafe` is confined to this module and `render.rs`: every `glow` call is `unsafe` because it
//! calls into the driver; callers only pass handles created by the same context.

use anyhow::{Result, bail};
use glow::HasContext;

use crate::assets::{Filter, TextureInfo, Wrap};

pub fn compile_program(
    gl: &glow::Context,
    vertex: &str,
    fragment: &str,
    attributes: &[&str],
) -> Result<glow::Program> {
    unsafe {
        let program = gl.create_program().map_err(anyhow::Error::msg)?;
        let mut shaders = Vec::new();
        for (kind, source) in [
            (glow::VERTEX_SHADER, vertex),
            (glow::FRAGMENT_SHADER, fragment),
        ] {
            let shader = gl.create_shader(kind).map_err(anyhow::Error::msg)?;
            gl.shader_source(shader, source);
            gl.compile_shader(shader);
            if !gl.get_shader_compile_status(shader) {
                bail!("shader compile error: {}", gl.get_shader_info_log(shader));
            }
            gl.attach_shader(program, shader);
            shaders.push(shader);
        }
        for (index, name) in attributes.iter().enumerate() {
            gl.bind_attrib_location(program, index as u32, name);
        }
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            bail!("program link error: {}", gl.get_program_info_log(program));
        }
        for shader in shaders {
            gl.detach_shader(program, shader);
            gl.delete_shader(shader);
        }
        Ok(program)
    }
}

/// Interleaved float vertex attributes + u32 index buffer.
pub struct GpuMesh {
    pub vao: glow::VertexArray,
    /// (byte offset, index count) per submesh.
    pub submeshes: Vec<(i32, i32)>,
}

fn f32_bytes(data: &[f32]) -> Vec<u8> {
    data.iter().flat_map(|v| v.to_le_bytes()).collect()
}

/// `layout` = component count of each attribute (attribute index = position in the slice).
pub fn upload_mesh(
    gl: &glow::Context,
    vertices: &[f32],
    layout: &[i32],
    submeshes: &[Vec<u32>],
) -> Result<GpuMesh> {
    let stride: i32 = layout.iter().sum::<i32>() * 4;
    let mut indices: Vec<u8> = Vec::new();
    let mut ranges = Vec::new();
    for submesh in submeshes {
        ranges.push((indices.len() as i32, submesh.len() as i32));
        indices.extend(submesh.iter().flat_map(|i| i.to_le_bytes()));
    }
    unsafe {
        let vao = gl.create_vertex_array().map_err(anyhow::Error::msg)?;
        gl.bind_vertex_array(Some(vao));
        let vbo = gl.create_buffer().map_err(anyhow::Error::msg)?;
        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &f32_bytes(vertices), glow::STATIC_DRAW);
        let ebo = gl.create_buffer().map_err(anyhow::Error::msg)?;
        gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
        gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, &indices, glow::STATIC_DRAW);
        let mut offset = 0;
        for (index, &size) in layout.iter().enumerate() {
            gl.enable_vertex_attrib_array(index as u32);
            gl.vertex_attrib_pointer_f32(index as u32, size, glow::FLOAT, false, stride, offset);
            offset += size * 4;
        }
        gl.bind_vertex_array(None);
        Ok(GpuMesh {
            vao,
            submeshes: ranges,
        })
    }
}

/// A vertex/index buffer pair refilled every draw (particles, trails).
pub struct DynamicMesh {
    vao: glow::VertexArray,
    vbo: glow::Buffer,
    ebo: glow::Buffer,
}

impl DynamicMesh {
    pub fn new(gl: &glow::Context, layout: &[i32]) -> Result<DynamicMesh> {
        let stride: i32 = layout.iter().sum::<i32>() * 4;
        unsafe {
            let vao = gl.create_vertex_array().map_err(anyhow::Error::msg)?;
            gl.bind_vertex_array(Some(vao));
            let vbo = gl.create_buffer().map_err(anyhow::Error::msg)?;
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            let ebo = gl.create_buffer().map_err(anyhow::Error::msg)?;
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(ebo));
            let mut offset = 0;
            for (index, &size) in layout.iter().enumerate() {
                gl.enable_vertex_attrib_array(index as u32);
                gl.vertex_attrib_pointer_f32(
                    index as u32,
                    size,
                    glow::FLOAT,
                    false,
                    stride,
                    offset,
                );
                offset += size * 4;
            }
            gl.bind_vertex_array(None);
            Ok(DynamicMesh { vao, vbo, ebo })
        }
    }

    /// Uploads `vertices` / `indices` and draws them as triangles with the bound program.
    pub fn draw(&self, gl: &glow::Context, vertices: &[f32], indices: &[u32]) {
        let index_bytes: Vec<u8> = indices.iter().flat_map(|i| i.to_le_bytes()).collect();
        unsafe {
            gl.bind_vertex_array(Some(self.vao));
            gl.bind_buffer(glow::ARRAY_BUFFER, Some(self.vbo));
            gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &f32_bytes(vertices), glow::STREAM_DRAW);
            gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, Some(self.ebo));
            gl.buffer_data_u8_slice(glow::ELEMENT_ARRAY_BUFFER, &index_bytes, glow::STREAM_DRAW);
            gl.draw_elements(glow::TRIANGLES, indices.len() as i32, glow::UNSIGNED_INT, 0);
        }
    }
}

fn gl_wrap(wrap: Wrap) -> i32 {
    (match wrap {
        Wrap::Repeat => glow::REPEAT,
        Wrap::Clamp => glow::CLAMP_TO_EDGE,
        Wrap::Mirror => glow::MIRRORED_REPEAT,
    }) as i32
}

/// Uploads RGBA8 pixels (rows bottom-up, as Unity/GL expect) with Unity's sampler settings.
/// Point → NEAREST(_MIPMAP_NEAREST), Bilinear → LINEAR(_MIPMAP_NEAREST), Trilinear → LINEAR_MIPMAP_LINEAR.
pub fn upload_texture(
    gl: &glow::Context,
    width: u32,
    height: u32,
    rgba: &[u8],
    info: &TextureInfo,
) -> Result<glow::Texture> {
    let (min, mag) = match (info.filter, info.mipmaps) {
        (Filter::Point, false) => (glow::NEAREST, glow::NEAREST),
        (Filter::Point, true) => (glow::NEAREST_MIPMAP_NEAREST, glow::NEAREST),
        (Filter::Bilinear, false) | (Filter::Trilinear, false) => (glow::LINEAR, glow::LINEAR),
        (Filter::Bilinear, true) => (glow::LINEAR_MIPMAP_NEAREST, glow::LINEAR),
        (Filter::Trilinear, true) => (glow::LINEAR_MIPMAP_LINEAR, glow::LINEAR),
    };
    unsafe {
        let texture = gl.create_texture().map_err(anyhow::Error::msg)?;
        gl.bind_texture(glow::TEXTURE_2D, Some(texture));
        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
        gl.tex_image_2d(
            glow::TEXTURE_2D,
            0,
            glow::RGBA8 as i32,
            width as i32,
            height as i32,
            0,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelUnpackData::Slice(Some(rgba)),
        );
        if info.mipmaps {
            gl.generate_mipmap(glow::TEXTURE_2D);
        }
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, min as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, mag as i32);
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, gl_wrap(info.wrap_u));
        gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, gl_wrap(info.wrap_v));
        Ok(texture)
    }
}

/// Reads the back buffer as top-down RGBA8 rows.
pub fn read_framebuffer(gl: &glow::Context, width: u32, height: u32) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    unsafe {
        gl.pixel_store_i32(glow::PACK_ALIGNMENT, 1);
        gl.read_pixels(
            0,
            0,
            width as i32,
            height as i32,
            glow::RGBA,
            glow::UNSIGNED_BYTE,
            glow::PixelPackData::Slice(Some(&mut pixels)),
        );
    }
    let row = (width * 4) as usize;
    let mut flipped = Vec::with_capacity(pixels.len());
    for y in (0..height as usize).rev() {
        flipped.extend_from_slice(&pixels[y * row..(y + 1) * row]);
    }
    flipped
}
