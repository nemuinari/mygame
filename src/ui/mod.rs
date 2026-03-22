pub mod background;
pub mod character;
pub mod messenger;

use bevy::prelude::*;
use messenger::TypewriterState;

pub struct UiModulePlugin;

impl Plugin for UiModulePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TypewriterState::default())
            .add_systems(Startup, setup_ui)
            .add_observer(messenger::on_scenario_text_changed)
            .add_systems(Update, messenger::update_typewriter)
            // 実行順を chain() で明示的に固定する:
            //   1. apply_pending_waiting_choice : Observer が立てたフラグを State に反映
            //   2. handle_choice_buttons        : クリックを検出してキューに積む
            //   3. advance_on_input             : キー入力 / クリック後の進行判定
            //   4. handle_choice_selected       : キューを消費して ScenarioState を更新
            .add_systems(
                Update,
                (
                    crate::scenario::apply_pending_waiting_choice,
                    messenger::handle_choice_buttons,
                    crate::scenario::advance_on_input,
                    crate::scenario::handle_choice_selected,
                )
                    .chain(),
            )
            .add_systems(Update, background::handle_background_change)
            .add_systems(Update, character::handle_character_show)
            .add_systems(Update, character::handle_character_hide);
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    background::spawn_background(&mut commands);
    character::spawn_character_slots(&mut commands);
    messenger::spawn_messenger(&mut commands, &asset_server);
}
