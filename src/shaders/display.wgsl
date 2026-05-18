// Display shader - Renders emulated Mac framebuffer to screen

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0)
var t_diffuse: texture_2d<f32>;

@group(0) @binding(1)
var s_diffuse: sampler;

// Vertex shader - Fullscreen quad
@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    // Generate fullscreen triangle strip (2 triangles = 6 vertices)
    let x = f32((vertex_index & 1u) << 1u) - 1.0;
    let y = 1.0 - f32((vertex_index & 2u));
    
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    
    return out;
}

// Fragment shader - Sample framebuffer texture
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(t_diffuse, s_diffuse, in.tex_coords);
}
