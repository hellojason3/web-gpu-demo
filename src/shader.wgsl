@group(0) @binding(0)
var<storage, read_write> data: array<f32>;

@compute @workgroup_size(4)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    data[index] = f32(index) * 2.0;
}