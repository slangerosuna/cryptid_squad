use crate::utils::math::matrix::Matrix4;
use crate::core::*;
use std::any::Any;

// component type 2
#[derive(Debug)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 3],
    pub scale: [f32; 3],
}
impl_component!(Transform, 2);

impl Transform {
    pub const fn new(
        position: [f32; 3],
        rotation: [f32; 3],
        scale: [f32; 3],
    ) -> Self {
        Self {
            position,
            rotation,
            scale,
        }
    }

    pub fn get_model(&self) -> Matrix4 {
        let translation = self.get_translation();
        let rotation = self.get_rotation();
        let scale = self.get_scale();

        translation * rotation * scale
    }

    pub const fn get_scale(&self) -> Matrix4 {
        let scale = self.scale;

        Matrix4([
            [scale[0], 0.0, 0.0, 0.0],
            [0.0, scale[1], 0.0, 0.0],
            [0.0, 0.0, scale[2], 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub const fn get_translation(&self) -> Matrix4 {
        let position = self.position;

        Matrix4([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [position[0], position[1], position[2], 1.0],
        ])
    }

    pub fn get_rotation(&self) -> Matrix4 {
        self.get_rotation_on_axis(0) * self.get_rotation_on_axis(1) * self.get_rotation_on_axis(2)
    }

    pub fn get_rotation_on_axis(&self, axis: usize) -> Matrix4 {
        let angle = self.rotation[axis];
        let axis = match axis {
            0 => [1.0, 0.0, 0.0],
            1 => [0.0, 1.0, 0.0],
            2 => [0.0, 0.0, 1.0],
            _ => panic!("Invalid axis"),
        };

        let (s, c) = angle.sin_cos();

        let x = axis[0];
        let y = axis[1];
        let z = axis[2];

        Matrix4([
            [
                c + (1.0 - c) * x * x,
                (1.0 - c) * x * y - s * z,
                (1.0 - c) * x * z + s * y,
                0.0,
            ],
            [
                (1.0 - c) * y * x + s * z,
                c + (1.0 - c) * y * y,
                (1.0 - c) * y * z - s * x,
                0.0,
            ],
            [
                (1.0 - c) * z * x - s * y,
                (1.0 - c) * z * y + s * x,
                c + (1.0 - c) * z * z,
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}
