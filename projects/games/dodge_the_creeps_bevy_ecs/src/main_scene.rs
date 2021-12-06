use crate::components::{GodotNode, Mob, Player, Spatial, Velocity};
use crate::events::{self, Action};
use crate::hud;
use crate::mob;
use crate::player;
use crate::resources::{Delta, ScreenSize};
use crate::systems::*;

use gdnative::api::{AnimatedSprite, PathFollow2D, Position2D, RigidBody2D};
use gdnative::prelude::*;
use rand::*;
use std::f64::consts::PI;
use rand::seq::SliceRandom;

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;

use mob::MOB_TYPES;

#[derive(NativeClass)]
#[inherit(Node2D)]
#[user_data(user_data::LocalCellData<Main>)]
pub struct Main {
    #[property]
    mob: Ref<PackedScene>,
    score: i64,
    schedule: Schedule,
    world: World,
}



#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    InGame,
}

const PRESSED_ACTIONS: &[&str] = &["ui_left", "ui_right", "ui_down", "ui_up"];

#[methods]
impl Main {
    fn new(_owner: &Node2D) -> Self {
        let mut builder = App::build();
        builder
            .add_state(AppState::MainMenu)
            .add_event::<events::InputEvent>()
            .add_system_to_stage(CoreStage::PreUpdate, process_player_movement.system())
            .add_system_set(
                SystemSet::on_enter(AppState::InGame).with_system(cleanup_mobs.system())
            )
            .add_system_set(
                SystemSet::on_update(AppState::InGame).with_system(movement.system())
            )
            .add_system_to_stage(CoreStage::PostUpdate, sync_entity.system());
        let App { schedule, mut world, .. } = builder.app;
        world.insert_resource(Delta::default());

        Main {
            mob: PackedScene::new().into_shared(),
            score: 0,
            schedule,
            world,
        }
    }

    #[export]
    fn _input(&mut self, _owner: &Node2D, event: Ref<InputEvent>) {
        let e = unsafe { event.assume_safe() };
        if !e.is_action_type () {
            return;
        }
        for action in PRESSED_ACTIONS {
            if e.is_action(action) {
                let mut events = self.world.get_resource_mut::<bevy_app::Events<events::InputEvent>>().unwrap();
                if e.is_pressed() {
                    events.send(events::InputEvent(Action::Pressed(action)));
                } else if !e.is_pressed() {
                    events.send(events::InputEvent(Action::Released(action)));
                }
            }
        }
    }

    #[export]
    fn _ready(&mut self, owner: &Node2D) {
        let player = unsafe {
            owner
                .get_node_as::<Node2D>("player")
                .unwrap()
                .claim()
        };
        let start_position = unsafe { owner.get_node_as::<Position2D>("start_position").unwrap() };
        self.world.spawn()
            .insert(Player)
            .insert(Spatial { position: start_position.position(), rotation: 0.0 } )
            .insert(Velocity(Vector2::default()))
            .insert(GodotNode(player));

        let viewport = owner.get_viewport_rect();
        self.world.insert_resource(ScreenSize(viewport.size));
    }

    #[export]
	fn _physics_process(&mut self, _owner: &Node2D, dt: f64) {
		self.world.clear_trackers();
		let mut delta = self.world.get_resource_mut::<Delta>()
			.expect("we just added SimDt in Ecs::new");
        delta.0 = dt as f32;
		self.schedule.run(&mut self.world);
        // todo: self.sim_schedule.run(&mut self.world);
	}

    /*
    fn _process(&mut self, _owner: &Node2D, dt: f64) {
        self.idle_schedule.run(&mut self.world);
    }
    */

    #[export]
    fn game_over(&mut self, owner: &Node2D) {
        let mut app_state = self.world.get_resource_mut::<State<AppState>>().expect("we just added SimDt in Ecs::new");
        app_state.set(AppState::MainMenu).unwrap();

        let score_timer = unsafe { owner.get_node_as::<Timer>("score_timer").unwrap() };
        let mob_timer = unsafe { owner.get_node_as::<Timer>("mob_timer").unwrap() };

        score_timer.stop();
        mob_timer.stop();

        let hud = unsafe { owner.get_node_as_instance::<hud::Hud>("hud").unwrap() };
        hud.map(|x, o| x.show_game_over(&*o))
            .ok()
            .unwrap_or_else(|| godot_print!("Unable to get hud"));
    }

    #[export]
    fn new_game(&mut self, owner: &Node2D) {
        let mut app_state = self.world.get_resource_mut::<State<AppState>>().expect("we just added SimDt in Ecs::new");
        app_state.set(AppState::InGame).unwrap();

        let player = unsafe {
            owner
                .get_node_as_instance::<player::Player>("player")
                .unwrap()
        };
        let start_timer = unsafe { owner.get_node_as::<Timer>("start_timer").unwrap() };

        self.score = 0;

        player
            .map(|x, o| x.start(&*o))
            .ok()
            .unwrap_or_else(|| godot_print!("Unable to get player"));

        start_timer.start(0.0);

        let hud = unsafe { owner.get_node_as_instance::<hud::Hud>("hud").unwrap() };
        hud.map(|x, o| {
            x.update_score(&*o, self.score);
            x.show_message(&*o, "Get Ready".into());
        })
        .ok()
        .unwrap_or_else(|| godot_print!("Unable to get hud"));
    }

    #[export]
    fn on_start_timer_timeout(&self, owner: &Node2D) {
        let mob_timer = unsafe { owner.get_node_as::<Timer>("mob_timer").unwrap() };
        let score_timer = unsafe { owner.get_node_as::<Timer>("score_timer").unwrap() };
        mob_timer.start(0.0);
        score_timer.start(0.0);
    }

    #[export]
    fn on_score_timer_timeout(&mut self, owner: &Node2D) {
        self.score += 1;

        let hud = unsafe { owner.get_node_as_instance::<hud::Hud>("hud").unwrap() };
        hud.map(|x, o| x.update_score(&*o, self.score))
            .ok()
            .unwrap_or_else(|| godot_print!("Unable to get hud"));
    }

    #[export]
    fn on_mob_timer_timeout(&mut self, owner: &Node2D) {
        let mob_spawn_location = unsafe {
            owner
                .get_node_as::<PathFollow2D>("mob_path/mob_spawn_locations")
                .unwrap()
        };

        let mob_instance: Ref<RigidBody2D, _> = instance_scene(&self.mob);

        let mut rng = rand::thread_rng();
        let offset = rng.gen_range(std::u32::MIN..std::u32::MAX);

        mob_spawn_location.set_offset(offset.into());

        let mut direction = mob_spawn_location.rotation() + PI / 2.0;

        mob_instance.set_position(mob_spawn_location.position());

        direction += rng.gen_range(-PI / 4.0..PI / 4.0);
        mob_instance.set_rotation(direction);
        mob_instance.set_linear_velocity(Vector2::new(rng.gen_range(150.0..250.0), 0.0));
        mob_instance.set_linear_velocity(mob_instance.linear_velocity().rotated(direction as f32));

        let mob_entity = mob_instance.into_shared();
        let mob = unsafe { mob_entity.assume_safe() };
        owner.add_child(mob, false);
        self.world.spawn()
            .insert(Mob)
            .insert(Spatial { position: mob.position(), rotation: mob.rotation() as f32 } )
            .insert(Velocity(mob.linear_velocity()))
            .insert(GodotNode(mob.upcast::<Node2D>().claim()));

        let mut rng = rand::thread_rng();
        let animated_sprite = unsafe {
            mob
                .get_node_as::<AnimatedSprite>("animated_sprite")
                .unwrap()
        };
        let animation = MOB_TYPES.choose(&mut rng).unwrap().to_str();
        animated_sprite.set_animation(animation);

        let hud = unsafe { owner.get_node_as_instance::<hud::Hud>("hud").unwrap() };

        hud.map(|_, o| {
            o.connect(
                "start_game",
                mob,
                "on_start_game",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();
        }).unwrap();
    }
}

/// Root here is needs to be the same type (or a parent type) of the node that you put in the child
///   scene as the root. For instance Spatial is used for this example.
fn instance_scene<Root>(scene: &Ref<PackedScene, Shared>) -> Ref<Root, Unique>
where
    Root: gdnative::object::GodotObject<Memory = ManuallyManaged> + SubClass<Node>,
{
    let scene = unsafe { scene.assume_safe() };

    let instance = scene
        .instance(PackedScene::GEN_EDIT_STATE_DISABLED)
        .expect("should be able to instance scene");

    let instance = unsafe { instance.assume_unique() };

    instance
        .try_cast::<Root>()
        .expect("root node type should be correct")
}

pub fn load_scene(path: &str) -> Option<Ref<PackedScene, ThreadLocal>> {
    let scene = ResourceLoader::godot_singleton().load(path, "PackedScene", false)?;
    let scene = unsafe { scene.assume_safe() };
  
    // scene.cast::<PackedScene>()
    return None;
}


