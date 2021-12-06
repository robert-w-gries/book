use gdnative::prelude::*;

pub struct GodotNode(pub Ref<Node2D>);

pub struct Spatial {
    pub position: Vector2,
    pub rotation: f32,
}

pub struct Mob;

pub struct Velocity(pub Vector2);

pub struct Player;