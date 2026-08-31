use cgmath::{vec2, Vector2, Vector3};

use anyhow::Result;

#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub orientation: Vector3<f32>, // Yaw, Pitch, Roll
    pub mouse_pos: Vector2<f64>,
}

impl Camera {
    pub fn get_look_at(&self) -> Vector3<f32> {
        Vector3 {
            x: f32::cos(self.orientation.x) * f32::cos(self.orientation.y),
            y: f32::sin(self.orientation.x) * f32::cos(self.orientation.y),
            z: f32::sin(self.orientation.y),
        }
    }
    pub fn update_camera(&mut self) -> Result<()> {
        self.orientation += Vector3 {
            x: self.mouse_pos.x as f32 * -1.0,
            y: self.mouse_pos.y as f32 * 1.0,
            z: 0.0,
        } * 0.003; // TODO: Sensibilty to be implemented
        self.mouse_pos = vec2(0.0, 0.0);

        Ok(())
    }
}
