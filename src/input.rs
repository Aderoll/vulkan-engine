use anyhow::Result;

use crate::{camera, App};

use winit::keyboard::Key;
use winit::{event::KeyEvent, keyboard::NamedKey};

pub fn get_keyboard_inputs(app: &mut App, event: &KeyEvent) -> Result<()> {
    if let Some(character) = event.logical_key.to_text() {
        match character {
            "w" => app.camera.position -= 0.1 * app.camera.get_look_at(),
            "s" => app.camera.position += 0.1 * app.camera.get_look_at(),
            _ => (),
        }
    } else if let Key::Named(key) = event.logical_key {
        match key {
            NamedKey::ArrowRight => app.camera.orientation.x -= 0.05,
            NamedKey::ArrowLeft => app.camera.orientation.x += 0.05,
            NamedKey::ArrowDown => app.camera.orientation.y -= 0.05,
            NamedKey::ArrowUp => app.camera.orientation.y += 0.05,
            _ => (),
        }
    }
    Ok(())
}
