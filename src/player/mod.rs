use bevy::{
    input::common_conditions::{input_just_pressed, input_just_released, input_pressed},
    prelude::*,
};
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::input::update_cursor_world_coords;

use kill::{reset_player_on_level_switch, reset_player_position, KillPlayerEvent};
use light::{handle_color_switch, preview_light_path, shoot_light, PlayerLightInventory};
use movement::{move_player, queue_jump, PlayerMovement};
use spawn::{add_player_sensors, init_player_bundle};

pub mod kill;
pub mod light;
pub mod movement;
mod spawn;

pub struct PlayerManagementPlugin;
impl Plugin for PlayerManagementPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<KillPlayerEvent>()
            .add_systems(Update, add_player_sensors) // ran when LDTK spawns the player
            .add_systems(FixedUpdate, move_player)
            .add_systems(
                Update,
                queue_jump
                    .run_if(input_just_pressed(KeyCode::Space))
                    .before(move_player),
            )
            .add_systems(Update, handle_color_switch)
            .add_systems(
                Update,
                (
                    preview_light_path.run_if(input_pressed(MouseButton::Left)),
                    shoot_light.run_if(input_just_released(MouseButton::Left)),
                )
                    .after(handle_color_switch)
                    .after(update_cursor_world_coords),
            )
            .add_systems(Update, reset_player_on_level_switch)
            .add_systems(Update, reset_player_position)
            .add_systems(
                Update,
                quick_reset.run_if(input_just_pressed(KeyCode::KeyR)),
            );
    }
}

/// To signal our own code to finish the initialization of the player (adding sensors, etc)
#[derive(Component, Default)]
pub struct PlayerMarker;

#[derive(Default, Bundle)]
pub struct PlayerBundle {
    body: RigidBody,
    controller: KinematicCharacterController,
    controller_output: KinematicCharacterControllerOutput,
    collider: Collider,
    collision_groups: CollisionGroups,
    friction: Friction,
    restitution: Restitution,
    player_movement: PlayerMovement,
    light_inventory: PlayerLightInventory,
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct LdtkPlayerBundle {
    player_marker: PlayerMarker,
    #[with(init_player_bundle)]
    player: PlayerBundle,
    #[sprite("lyra.png")]
    sprite: Sprite,
    #[worldly]
    worldly: Worldly,
    #[from_entity_instance]
    instance: EntityInstance,
}

fn quick_reset(mut ev_kill_player: EventWriter<KillPlayerEvent>) {
    ev_kill_player.send(KillPlayerEvent);
}
