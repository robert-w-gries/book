use gdnative::prelude::*;

mod components;
mod events;
mod hud;
mod main_scene;
mod mob;
mod player;
mod resources;
mod systems;

fn init(handle: InitHandle) {
    handle.add_class::<player::Player>();
    handle.add_class::<main_scene::Main>();
    handle.add_class::<hud::Hud>();
}

godot_init!(init);
