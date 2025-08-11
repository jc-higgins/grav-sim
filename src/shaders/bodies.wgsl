struct VsIn {
    @location(0) local_pos: vec2f,     // quad vertex [-1,1]
    @location(1) instance_pos: vec2f,  // body center in clip space
};
struct VsOut {
    @builtin(position) pos: vec4f,
    @location(0) uv: vec2f,            // pass local_pos to fragment
};

@vertex
fn vs(in: VsIn) -> VsOut {
    var out: VsOut;
    let radius = 0.02; // tweak size
    let pos = in.instance_pos + in.local_pos * radius;
    out.pos = vec4f(pos, 0.0, 1.0);
    out.uv = in.local_pos;
    return out;
}

@fragment
fn fs(in: VsOut) -> @location(0) vec4f {
    // circle mask in the quad
    let r = length(in.uv);
    if (r > 1.0) { discard; }
    return vec4f(0.9, 0.9, 0.9, 1.0);
}