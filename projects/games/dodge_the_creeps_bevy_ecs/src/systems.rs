use crate::components::{GodotNode, Mob, Player, Spatial, Speed, Velocity};
use crate::events::{Action, InputEvent};
use crate::resources::{Delta, ScreenSize};

use gdnative::api::{AnimatedSprite, Position2D};
use gdnative::prelude::{NodeExt, Vector2};

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_ecs::component::Component;

pub fn movement(screen_size: Res<ScreenSize>, delta: Res<Delta>, mut query: Query<(&mut Spatial, &Velocity, Option<&Player>)>) {
    for (mut spat, vel, is_player) in query.iter_mut() {
        spat.position.x += vel.0.x * delta.0;
        spat.position.y += vel.0.y * delta.0;
        if let Some(_) = is_player {
            spat.position.x = spat.position.x.max(0.0).min(screen_size.0.x);
            spat.position.y = spat.position.y.max(0.0).min(screen_size.0.y);
        }
    }
}

pub fn sync_entity(
    screen_size: Res<ScreenSize>,
    mut commands: Commands,
    mut q: Query<(Entity, &Spatial, &GodotNode, Option<&Mob>)>,
) {
    for (e, spat, godot, is_mob) in q.iter_mut() {
        let node = unsafe { godot.0.assume_safe() };
        if let Some(_) = is_mob {
            let Vector2 { x: pos_x, y: pos_y } = spat.position;
            let Vector2 { x: size_x, y: size_y } = screen_size.0;
            if pos_x < 0.0 || pos_x > size_x + 0.0 || pos_y < 0.0 || pos_y > size_y + 0.0 {
                commands.entity(e).despawn();
                node.queue_free();
                continue;
            }
        }

        node.set_position(spat.position);
        //node.set_rotation(spat.rotation as f64);
    }
}

pub fn reset_player_position(mut q: Query<(&mut Spatial, &mut GodotNode), With<Player>>) {
    let (mut spatial, godot_node) = q.single_mut().expect("There should always be exactly one player in the game!");
    let start_position = unsafe {
        let node = godot_node.0.assume_safe();
        node.get_node_as::<Position2D>("start_position").unwrap()
    };
    spatial.position = start_position.position();
}

pub fn set_player_animation(mut q: Query<(&Velocity, &GodotNode, &Speed), With<Player>>) {
    let (player_velocity, godot_node, speed) = q.single_mut().expect("There should always be exactly one player in the game!");
    let mut velocity = player_velocity.0;
    let animated_sprite = unsafe {
        let node = godot_node.0.assume_safe();
        node.get_node_as::<AnimatedSprite>("animated_sprite")
            .unwrap()
    };

    if velocity.length() > 0.0 {
        velocity = velocity.normalized() * Vector2::new(speed.0, speed.0);

        let animation;

        if velocity.x != 0.0 {
            animation = "right";

            animated_sprite.set_flip_v(false);
            animated_sprite.set_flip_h(velocity.x < 0.0)
        } else {
            animation = "up";

            animated_sprite.set_flip_v(velocity.y > 0.0)
        }

        animated_sprite.play(animation, false);
    } else {
        animated_sprite.stop();
    }
}

pub fn process_player_movement(
    mut input_events: EventReader<InputEvent>,
    mut q: Query<(&mut Velocity, &Speed), With<Player>>,
) {
    let (mut player_velocity, speed) = q.single_mut().expect("There should always be exactly one player in the game!");
    for event in input_events.iter() {
        let Vector2 { x: velocity_x, y: velocity_y } = player_velocity.0;
        if let Action::Released(action) = event.0 {
            match action {
                "ui_right" => {
                    player_velocity.0.x = velocity_x - speed.0;
                },
                "ui_left" => {
                    player_velocity.0.x = velocity_x + speed.0;
                },
                "ui_down" => {
                    player_velocity.0.y = velocity_y - speed.0;
                },
                "ui_up" => {
                    player_velocity.0.y = velocity_y + speed.0;
                },
                _ => (),
            };
        } else if let Action::Pressed(action) = event.0 {
            match action {
                "ui_right" => {
                    player_velocity.0.x = velocity_x + speed.0;
                },
                "ui_left" => {
                    player_velocity.0.x = velocity_x - speed.0;
                },
                "ui_down" => {
                    player_velocity.0.y = velocity_y + speed.0;
                },
                "ui_up" => {
                    player_velocity.0.y = velocity_y - speed.0;
                },
                _ => (),
            };
        }
    }
}

pub fn cleanup_mobs(
    mut commands: Commands,
    mut q: Query<(Entity, &GodotNode), With<Mob>>,
) {
    for (e, godot) in q.iter_mut() {
        let node = unsafe { godot.0.assume_safe() };
        commands.entity(e).despawn();
        node.queue_free();
    }
}

pub fn cleanup_system<T: Component>(
    mut commands: Commands,
    q: Query<Entity, With<T>>,
) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}
