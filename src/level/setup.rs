use crate::config::Config;
use crate::player::kill::KillPlayerEvent;
use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;

use super::CurrentLevel;

pub struct LevelSetupPlugin;

impl Plugin for LevelSetupPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_level);
    }
}

fn setup_level(mut commands: Commands, asset_server: Res<AssetServer>, config: Res<Config>, mut ev: EventWriter<KillPlayerEvent>) {
    let level_selection = LevelSelection::index(config.level_config.level_index);
    commands.insert_resource(level_selection);
    commands.insert_resource(CurrentLevel("14c704f0-c210-11ef-833b-5533c9bd8e92".into()));
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load(&config.level_config.level_path).into(),
        ..Default::default()
    });
    ev.send(KillPlayerEvent);
}
