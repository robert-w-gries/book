pub struct InputEvent(pub Action);

pub enum Action {
    Pressed(&'static str),
    Released(&'static str),
}

pub const PRESSED_ACTIONS: &[&str] = &["ui_left", "ui_right", "ui_down", "ui_up"];
