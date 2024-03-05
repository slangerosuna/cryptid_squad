use crate::core::*;
use std::any::Any;

// component type 3
#[derive(Debug)]
pub struct Camera {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub fov: f32,
    pub aspect_ratio: f32,
    pub near: f32,
    pub far: f32,
}
impl_component!(Camera, 3);

impl Camera {
    pub const fn new(
        position: [f32; 3],
        rotation: [f32; 3],
        fov: f32,
        aspect_ratio: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self {
            position,
            rotation,
            fov,
            aspect_ratio,
            near,
            far,
        }
    }

    pub fn get_proj(&self) -> [[f32; 4]; 4] {
        let fov = self.fov.to_radians();
        let aspect_ratio = self.aspect_ratio;
        let near = self.near;
        let far = self.far;

        let f = 1.0 / (fov / 2.0).tan();

        [
            [f / aspect_ratio, 0.0, 0.0, 0.0],
            [0.0, f, 0.0, 0.0],
            [0.0, 0.0, (far + near) / (near - far), -1.0],
            [0.0, 0.0, (2.0 * far * near) / (near - far), 0.0]
        ]
    }

    pub fn get_view(&self) -> [[f32; 4]; 4] {
        let position = self.position;
        let rotation = self.rotation;

        let cos = rotation[0].cos() * rotation[1].cos();
        let sin = rotation[0].sin() * rotation[1].cos();

        let x = cos * rotation[2].cos() - sin * rotation[2].sin();
        let y = sin * rotation[2].cos() + cos * rotation[2].sin();
        let z = rotation[0].sin();

        [
            [x, y, z, 0.0],
            [-y, x, z, 0.0],
            [-z, -z, rotation[0].cos(), 0.0],
            [position[0], position[1], position[2], 1.0],
        ]
    }
}
