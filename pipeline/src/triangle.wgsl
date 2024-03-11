// 顶点着色器
struct VertexInput {
    @location(0) position: vec3f,
    @location(1) color: vec3f,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4f,
    @location(0) color: vec3f,
};

@vertex
fn vs_main(
  in: VertexInput
) -> VertexOutput {
  var out: VertexOutput;
  out.color = in.color;
  out.clip_position = vec4f(in.position, 1.0);
  return out;
}

// 片元着色器

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4f {
    return vec4f(in.color, 1.0);
}
