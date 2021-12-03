use crate::hud;
use crate::mob;
use crate::player;
use gdnative::api::{AnimatedSprite, PathFollow2D, Position2D, RigidBody2D};
use gdnative::prelude::*;
use rand::*;
use std::f64::consts::PI;
use rand::seq::SliceRandom;

use bevy_ecs::prelude::*;
use bevy_ecs::component::Component;

use mob::{MOB_TYPES, MobType};

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


#[derive(Default)]
pub struct Delta(f32);

pub struct ScreenSize(Vector2);

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct Update;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
struct PostUpdate;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    InGame,
}

#[methods]
impl Main {
    fn new(_owner: &Node2D) -> Self {
        let mut schedule = Schedule::default();
        
        schedule.add_stage(Update, SystemStage::parallel()
            .with_system_set(State::<AppState>::get_driver())
            .with_system_set(
                SystemSet::on_enter(AppState::InGame)
                    .with_system(cleanup_mobs.system())
            )
            .with_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(movement.system())
            )
        );
        schedule.add_stage(PostUpdate, SystemStage::single_threaded().with_system(sync_entity.system()));

        let mut world = World::default();
        world.insert_resource(Delta::default());

        let window_size = gdnative::api::OS::godot_singleton().window_size();
        world.insert_resource(ScreenSize(window_size));

        world.insert_resource(State::new(AppState::MainMenu));

        Main {
            mob: PackedScene::new().into_shared(),
            score: 0,
            schedule,
            world,
        }
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

        let start_position = unsafe { owner.get_node_as::<Position2D>("start_position").unwrap() };
        let player = unsafe {
            owner
                .get_node_as_instance::<player::Player>("player")
                .unwrap()
        };
        let start_timer = unsafe { owner.get_node_as::<Timer>("start_timer").unwrap() };

        self.score = 0;

        player
            .map(|x, o| x.start(&*o, start_position.position()))
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

        let mob_scene: Ref<RigidBody2D, _> = instance_scene(&self.mob);

        let mut rng = rand::thread_rng();
        let offset = rng.gen_range(std::u32::MIN..std::u32::MAX);

        mob_spawn_location.set_offset(offset.into());

        let mut direction = mob_spawn_location.rotation() + PI / 2.0;

        mob_scene.set_position(mob_spawn_location.position());

        direction += rng.gen_range(-PI / 4.0..PI / 4.0);
        mob_scene.set_rotation(direction);
        mob_scene.set_linear_velocity(Vector2::new(rng.gen_range(150.0..250.0), 0.0));
        mob_scene.set_linear_velocity(mob_scene.linear_velocity().rotated(direction as f32));

        let mob_entity = mob_scene.into_shared();
        let mob_scene = unsafe { mob_entity.assume_safe() };
        owner.add_child(mob_scene, false);
        self.world.spawn()
            .insert(Spatial { position: mob_scene.position(), rotation: mob_scene.rotation() as f32 } )
            .insert(Velocity(mob_scene.linear_velocity()))
            .insert(GodotNode(mob_entity));

        let mut rng = rand::thread_rng();
        let animated_sprite = unsafe {
            owner
                .get_node_as::<AnimatedSprite>("animated_sprite")
                .unwrap()
        };
        animated_sprite.set_animation(MOB_TYPES.choose(&mut rng).unwrap().to_str());

        let hud = unsafe { owner.get_node_as_instance::<hud::Hud>("hud").unwrap() };

        hud.map(|_, o| {
            o.connect(
                "start_game",
                mob_scene,
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

pub struct GodotNode(Ref<RigidBody2D>);

pub struct Spatial {
    pub position: Vector2,
    pub rotation: f32,
}

pub struct Velocity(Vector2);

fn movement(delta: Res<Delta>, mut query: Query<(&mut Spatial, &mut Velocity)>) {
    for (mut spat, mut vel) in query.iter_mut() {
        spat.position.x += vel.0.x * delta.0;
        spat.position.y += vel.0.y * delta.0;
    }
}

pub fn sync_entity(
    screen_size: Res<ScreenSize>,
    mut commands: Commands,
    mut q: Query<(Entity, &Spatial, &mut GodotNode)>,
) {
    for (e, spat, mut godot) in q.iter_mut() {
        let node = unsafe { godot.0.assume_safe() };
        let Vector2 { x: pos_x, y: pos_y } = spat.position;
        let Vector2 { x: size_x, y: size_y } = screen_size.0;

        if pos_x < 0.0 || pos_x > size_x + 0.0 || pos_y < 0.0 || pos_y > size_y + 0.0  {
            commands.entity(e).despawn();
            node.queue_free();
            continue;
        }

        node.set_position(spat.position);
        node.set_rotation(spat.rotation as f64);
    }
}

fn cleanup_mobs(
    mut commands: Commands,
    mut q: Query<(Entity, &mut GodotNode)>,
) {
    for (e, mut godot) in q.iter_mut() {
        let node = unsafe { godot.0.assume_safe() };
        commands.entity(e).despawn();
        node.queue_free();
    }
}

fn cleanup_system<T: Component>(
    mut commands: Commands,
    q: Query<Entity, With<T>>,
) {
    for e in q.iter() {
        commands.entity(e).despawn();
    }
}
