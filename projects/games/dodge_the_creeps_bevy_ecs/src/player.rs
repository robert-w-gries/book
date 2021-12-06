use gdnative::api::{AnimatedSprite, Area2D, CollisionShape2D, PhysicsBody2D};
use gdnative::prelude::*;

/// The player "class"
#[derive(NativeClass)]
#[inherit(Area2D)]
#[user_data(user_data::MutexData<Player>)]
#[register_with(Self::register_player)]
pub struct Player {
    #[property(default = 400.0)]
    speed: f32,

    screen_size: Vector2,
}

#[methods]
impl Player {
    fn register_player(builder: &ClassBuilder<Self>) {
        builder.add_signal(Signal {
            name: "hit",
            args: &[],
        });
    }

    fn new(_owner: &Area2D) -> Self {
        Player {
            speed: 400.0,
            screen_size: Vector2::new(0.0, 0.0),
        }
    }

    #[export]
    fn _ready(&mut self, owner: &Area2D) {
        let viewport = owner.get_viewport_rect();
        self.screen_size = viewport.size;
        owner.hide();
    }

    #[export]
    fn _process(&mut self, owner: &Area2D, delta: f32) {

    }

    #[export]
    fn on_player_body_entered(&self, owner: &Area2D, _body: Ref<PhysicsBody2D>) {
        owner.hide();
        owner.emit_signal("hit", &[]);

        let collision_shape = unsafe {
            owner
                .get_node_as::<CollisionShape2D>("collision_shape_2d")
                .unwrap()
        };

        collision_shape.set_deferred("disabled", true);
    }

    #[export]
    pub fn start(&self, owner: &Area2D) {
        owner.show();

        let collision_shape = unsafe {
            owner
                .get_node_as::<CollisionShape2D>("collision_shape_2d")
                .unwrap()
        };

        collision_shape.set_disabled(false);
    }
}
