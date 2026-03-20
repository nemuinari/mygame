pub mod background;
pub mod messenger;

use bevy::prelude::*;

pub struct UiModulePlugin;

impl Plugin for UiModulePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_ui)
            .add_observer(messenger::on_scenario_text_changed);
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    // 2D カメラ
    commands.spawn(Camera2d);
    background::spawn_background(&mut commands);
    messenger::spawn_messenger(&mut commands, &asset_server);
}
