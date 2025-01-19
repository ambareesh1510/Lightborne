use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::level::{misc::StartFlag, CurrentLevel, LevelSwitchEvent};

use super::{light::PlayerLightInventory, movement::PlayerMovement, PlayerMarker};

#[derive(Event)]
pub struct KillPlayerEvent;

pub fn reset_player_position(
    mut q_player: Query<&mut Transform, With<PlayerMarker>>,
    mut ev_kill_events: EventReader<KillPlayerEvent>,
    mut ev_level_switch: EventWriter<LevelSwitchEvent>,
    q_start_flag: Query<(&StartFlag, &EntityInstance)>,
    current_level: Res<CurrentLevel>,
) {
    let Ok(mut transform) = q_player.get_single_mut() else {
        return;
    };

    if q_start_flag.is_empty() {
        return;
    }

    if ev_kill_events.is_empty() {
        return;
    }
    ev_kill_events.clear();

    // Trigger level switch to reset the beams and stuff
    ev_level_switch.send(LevelSwitchEvent);


    for (flag, instance) in q_start_flag.iter() {
        if current_level.0 == flag.level_iid {
            transform.translation.x =
                instance.world_x.expect("Lightborne uses Free world layout") as f32;
            transform.translation.y =
                -instance.world_y.expect("Lightborne uses Free world layout") as f32;
            return;
        }
    }

    panic!("Couldn't find start flag to respawn at");
}

pub fn reset_player_on_level_switch(
    mut q_player: Query<(&mut PlayerMovement, &mut PlayerLightInventory), With<PlayerMarker>>,
    mut ev_level_switch: EventReader<LevelSwitchEvent>,
) {
    let Ok((mut movement, mut inventory)) = q_player.get_single_mut() else {
        return;
    };

    if ev_level_switch.is_empty() {
        return;
    }
    ev_level_switch.clear();

    *movement = PlayerMovement::default();
    *inventory = PlayerLightInventory::default();
}
