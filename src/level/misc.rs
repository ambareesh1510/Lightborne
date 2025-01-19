use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use crate::light::sensor::LightSensorBundle;

#[derive(Default, Component)]
pub struct ButtonMarker;

#[derive(Default, Bundle, LdtkEntity)]
pub struct ButtonBundle {
    marker: ButtonMarker,
    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[from_entity_instance]
    light_sensor: LightSensorBundle,
}

#[derive(Default, Component)]
pub struct StartMarker;

#[derive(Default, Component)]
pub struct StartFlag {
    pub level_iid: String,
}

#[derive(Default, Bundle, LdtkEntity)]
pub struct StartFlagBundle {
    marker: StartMarker,
    #[from_entity_instance]
    instance: EntityInstance,
}

pub fn init_start_marker(
    mut commands: Commands,
    q_start_flag: Query<(Entity, &Parent), Added<StartMarker>>,
    q_parent: Query<&Parent, Without<StartMarker>>,
    q_level: Query<&LevelIid>,
) {
    for (entity, parent) in q_start_flag.iter() {
        let Ok(level_entity) = q_parent.get(parent.get()) else {
            continue;
        };
        let Ok(level_iid) = q_level.get(level_entity.get()) else {
            continue;
        };
        commands.entity(entity).insert(StartFlag {
            level_iid: level_iid.to_string(),
        });
    }
}
