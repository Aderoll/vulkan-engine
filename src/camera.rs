use cgmath::Vector3;

#[derive(Clone, Debug)]
pub struct Camera {
    pub position: Vector3<f32>,
    pub orientation: Vector3<f32>, // Yaw, Pitch, Roll
}

impl Camera {
    pub fn get_look_at(&self) -> Vector3<f32> {
        Vector3 {
            x: f32::cos(self.orientation.x) * f32::cos(self.orientation.y),
            y: f32::sin(self.orientation.x) * f32::cos(self.orientation.y),
            z: f32::sin(self.orientation.y),
        }
    }
}
