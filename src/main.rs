mod scenario;
mod ui;

use bevy::prelude::*;
use scenario::ScenarioPlugin;
use ui::UiModulePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Bevy Visual Novel".into(),
                ..default()
            }),
            ..default()
        }))
        // シナリオプラグイン（UI より先に登録して Resource を確保する）
        .add_plugins(ScenarioPlugin {
            initial_scene_path: "assets/scenarios/chapter_01/scene_01.ron".into(),
        })
        .add_plugins(UiModulePlugin)
        .run();
}
