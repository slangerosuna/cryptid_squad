pub fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    let len = len.sqrt();
    [v[0] / len, v[1] / len, v[2] / len]
}
