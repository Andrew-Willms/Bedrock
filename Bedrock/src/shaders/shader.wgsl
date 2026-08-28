struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
};

@vertex
fn main_vertex_shader(input: VertexInput) -> VertexOutput {

    var output: VertexOutput;

    output.position = vec4<f32>(
        input.position,
        0.0,
        1.0
    );

    return output;
}

@fragment
fn main_fragment_shader() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 1.0, 1.0, 1.0);
}