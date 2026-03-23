pub mod background;
pub mod character;
pub mod messenger;
pub mod system_buttons;

use bevy::prelude::*;
use messenger::TypewriterState;

pub struct UiModulePlugin;

impl Plugin for UiModulePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TypewriterState::default())
            .add_systems(Startup, setup_ui)
            .add_observer(messenger::on_scenario_text_changed)
            .add_systems(
                Update,
                (
                    crate::scenario::apply_pending_waiting_choice,
                    messenger::handle_choice_buttons,
                    crate::scenario::advance_on_input,
                    crate::scenario::handle_choice_selected,
                    messenger::update_typewriter,
                )
                    .chain(),
            )
            .add_systems(Update, background::handle_background_change)
            .add_systems(Update, character::handle_character_show)
            .add_systems(Update, character::handle_character_hide)
            // システムボタン
            .add_systems(
                Update,
                (
                    system_buttons::handle_tab_button,
                    system_buttons::animate_system_bar,
                    system_buttons::handle_save_button,
                    system_buttons::handle_load_button,
                    system_buttons::sync_modal,
                    system_buttons::handle_modal_buttons,
                    system_buttons::auto_save_on_advance,
                )
                    .chain(),
            );
    }
}

fn setup_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);
    background::spawn_background(&mut commands);
    character::spawn_character_slots(&mut commands);
    messenger::spawn_messenger(&mut commands, &asset_server);
    system_buttons::spawn_system_bar(&mut commands, &asset_server);
}
