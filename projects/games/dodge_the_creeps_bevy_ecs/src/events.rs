pub struct InputEvent(pub Action);

pub enum Action {
    Pressed(&'static str),
    Released(&'static str),
}
