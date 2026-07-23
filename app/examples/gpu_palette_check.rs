//! GPU palette-shader regression check: draw a known 0..255 R-gradient value
//! texture through (a) the palette material and (b) a passthrough grayscale
//! material, capture the framebuffer, and assert every column matches the CPU
//! LUT entry exactly. Catches any value-transfer corruption in the GPU path —
//! sRGB/gamma shifts, sampler mixups, or vertex-color scaling bugs (e.g. the
//! unnormalized `color0` Byte4 attribute that once saturated every palette
//! color to pure primaries; see the `color0 / 255.0` comment in `scope.rs`).
//!
//! Needs a GL context (opens a brief window), so it lives as an example, not a
//! `cargo test`. Exits 0 on PASS, 1 on mismatch.
//!
//! Usage: cargo run -p rustywx --example gpu_palette_check --release

#[allow(unused_imports)]
use ply_engine::prelude::*; // brings the `macroquad` re-export into scope

use macroquad::material::{
    MaterialParams, gl_use_default_material, gl_use_material, load_material,
};
use macroquad::math::Vec2;
use macroquad::miniquad::ShaderSource;
use macroquad::texture::{DrawTextureParams, FilterMode, Texture2D, draw_texture_ex};
use macroquad::window::{clear_background, next_frame, screen_width};

use rustywx::colors;
use rustywx::scope;

const GRAY_FRAGMENT: &str = r#"#version 100
precision mediump float;
varying vec2 uv;
varying vec4 color;
uniform sampler2D Texture;
void main() {
    vec4 v = texture2D(Texture, uv);
    gl_FragColor = vec4(v.r, v.r, v.r, 1.0);
}"#;

const GRAY_VERTEX: &str = r#"#version 100
attribute vec3 position;
attribute vec2 texcoord;
attribute vec4 color0;
varying lowp vec2 uv;
varying lowp vec4 color;
uniform mat4 Model;
uniform mat4 Projection;
void main() {
    gl_Position = Projection * Model * vec4(position, 1);
    color = color0;
    uv = texcoord;
}"#;

#[macroquad::main("dbg_transfer")]
async fn main() {
    // 256x1 gradient value texture: R = x, A = 255.
    let mut grad = [0u8; 256 * 4];
    for i in 0..256 {
        grad[i * 4] = i as u8;
        grad[i * 4 + 3] = 255;
    }
    let grad_tex = Texture2D::from_rgba8(256, 1, &grad);
    grad_tex.set_filter(FilterMode::Nearest); // exact texels, no interpolation

    let lut = Texture2D::from_rgba8(256, 1, &colors::dbz_lut());
    lut.set_filter(FilterMode::Nearest);
    let palette_mat = scope::load_palette_material();
    palette_mat.set_texture("Palette", lut.clone());

    let gray_mat = load_material(
        ShaderSource::Glsl {
            vertex: GRAY_VERTEX,
            fragment: GRAY_FRAGMENT,
        },
        MaterialParams::default(),
    )
    .unwrap();

    for frame in 0..5 {
        clear_background(macroquad::color::BLACK);
        // Row 1 (y 0..50): gradient through palette material.
        gl_use_material(&palette_mat);
        draw_texture_ex(
            &grad_tex,
            0.0,
            0.0,
            macroquad::color::WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(512.0, 50.0)),
                ..Default::default()
            },
        );
        // Row 2 (y 60..110): gradient through grayscale passthrough.
        gl_use_material(&gray_mat);
        draw_texture_ex(
            &grad_tex,
            0.0,
            60.0,
            macroquad::color::WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(512.0, 50.0)),
                ..Default::default()
            },
        );
        gl_use_default_material();
        // Row 3 (y 120..170): gradient drawn with NO material (default pipeline)
        // — shows the raw value texture as the fixed pipeline sees it.
        draw_texture_ex(
            &grad_tex,
            0.0,
            120.0,
            macroquad::color::WHITE,
            DrawTextureParams {
                dest_size: Some(Vec2::new(512.0, 50.0)),
                ..Default::default()
            },
        );
        if frame == 4 {
            let img = macroquad::texture::get_screen_data();
            img.export_png("/tmp/gpu_palette_check.png");
            // get_screen_data returns the raw GL readback (bottom-up rows);
            // flip y when indexing. Sample row centers: y=25 palette, y=85
            // gray (in top-down draw coordinates).
            let h = img.height as u32;
            let lut_cpu = colors::dbz_lut();
            let mut failures = 0;
            println!("col  in    palette_rgb      expect_lut       gray");
            for col in 0..256usize {
                let x = (col * 2 + 1) as u32; // 512px wide / 256 entries
                let p = img.get_pixel(x, h - 1 - 25);
                let g = img.get_pixel(x, h - 1 - 85);
                let e = &lut_cpu[col * 4..col * 4 + 4];
                // Alpha-blended over black: expected on-screen RGB is
                // lut_rgb * lut_a / 255.
                let exp: Vec<i32> = (0..3)
                    .map(|ch| (e[ch] as i32 * e[3] as i32) / 255)
                    .collect();
                let got = [
                    (p.r * 255.0).round() as i32,
                    (p.g * 255.0).round() as i32,
                    (p.b * 255.0).round() as i32,
                ];
                let gray = (g.r * 255.0).round() as i32;
                let ok = (0..3).all(|ch| (got[ch] - exp[ch]).abs() <= 2)
                    && (gray - col as i32).abs() <= 1;
                if !ok {
                    failures += 1;
                }
                if col % 16 == 0 || !ok {
                    println!(
                        "{col:3}  {:3}  ({:3},{:3},{:3})  ({:3},{:3},{:3})  {gray:3}  {}",
                        col,
                        got[0],
                        got[1],
                        got[2],
                        exp[0],
                        exp[1],
                        exp[2],
                        if ok { "" } else { "  <-- MISMATCH" }
                    );
                }
            }
            println!("wrote /tmp/gpu_palette_check.png");
            if failures > 0 {
                println!("FAIL: {failures}/256 palette entries mismatched");
                std::process::exit(1);
            }
            println!("PASS: GPU palette transfer matches CPU LUT for all 256 entries");
        }
        let _ = screen_width();
        next_frame().await;
    }
    std::process::exit(0);
}
