pub fn calculate(x: f32, y: f32, z: f32, x2: f32, y2: f32, z2: f32) -> f32 {
    let dx = x2 - x;
    let dy = y2 - y;
    let dz = z2 - z;

    (dx * dx + dy * dy + dz * dz).sqrt()
}
