use anyhow::Result;
use cgmath::{vec2, vec3};
use winit::dpi::PhysicalPosition;
use winit::event;

use crate::App;

use winit::keyboard::KeyCode::*;

pub fn get_inputs<T>(app: &mut App, event: &event::Event<T>) -> Result<()> {
    if !app.input_helper.update(event) {
        return Ok(());
    };
    Ok(())
}
pub fn mouse_moved(app: &mut App, movement: PhysicalPosition<f64>) -> Result<()> {
    let new = vec2(movement.x, movement.y);
    let delta = new - app.last_mouse_pos;
    app.last_mouse_pos = new;
    app.camera.mouse_pos += delta;
    Ok(())
}
pub fn process_inputs(app: &mut App) -> Result<()> {
    let frame_time = app.last_frame.elapsed().as_secs_f32();

    if app.input_helper.key_held(KeyW) {
        app.camera.position += frame_time * app.camera.get_look_at() * 5.0;
    }
    if app.input_helper.key_held(KeyS) {
        app.camera.position -= frame_time * app.camera.get_look_at() * 5.0;
    }
    if app.input_helper.key_held(KeyA) {
        let lavec = app.camera.get_look_at();
        let direction_vector = vec3(lavec.y, lavec.x * -1.0, 0.0);
        app.camera.position -= frame_time * direction_vector * 5.0;
    }
    if app.input_helper.key_held(KeyD) {
        let lavec = app.camera.get_look_at();
        let direction_vector = vec3(lavec.y * -1.0, lavec.x, 0.0);
        app.camera.position -= frame_time * direction_vector * 5.0;
    }
    // Camera
    if app.input_helper.key_held(ArrowRight) {
        app.camera.orientation.x -= frame_time;
    }
    if app.input_helper.key_held(ArrowLeft) {
        app.camera.orientation.x += frame_time;
    }
    if app.input_helper.key_held(ArrowUp) {
        app.camera.orientation.y += frame_time;
    }
    if app.input_helper.key_held(ArrowDown) {
        app.camera.orientation.y -= frame_time;
    }
    Ok(())
}
