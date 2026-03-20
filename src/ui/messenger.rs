use crate::scenario::types::Step;
use crate::scenario::{ScenarioState, ScenarioTextChanged};
use bevy::prelude::*;
use bevy::text::FontSize;

// ────────────────────────────────────────────────
// Components
// ────────────────────────────────────────────────

#[derive(Component)]
pub struct MessageText;

#[derive(Component)]
pub struct SpeakerText;

// ────────────────────────────────────────────────
// Spawn
// ────────────────────────────────────────────────

pub fn spawn_messenger(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let font = asset_server.load("fonts/NotoSansJP-Regular.ttf");

    commands
        .spawn((
            Name::new("Message_Window_Root"),
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(90.0),
                height: Val::Percent(22.0),
                left: Val::Percent(5.0),
                bottom: Val::Percent(3.0),
                padding: UiRect::axes(Val::Percent(2.5), Val::Percent(2.0)),
                border: UiRect::all(Val::Percent(0.3)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Percent(1.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.15, 0.92)),
            BorderColor::from(Color::srgba(0.6, 0.6, 1.0, 0.8)),
        ))
        .with_children(|frame| {
            // 話者名
            frame.spawn((
                SpeakerText,
                Name::new("Speaker_Text"),
                Text::new(""),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.9, 1.0, 1.0)),
            ));

            // 本文
            frame.spawn((
                MessageText,
                Name::new("Message_Text"),
                Text::new(""),
                TextFont {
                    font: font.clone().into(),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
}

// ────────────────────────────────────────────────
// Observer（Bevy 0.17+）
// ────────────────────────────────────────────────

/// ScenarioTextChanged を Observer で受け取り、話者名・本文を書き換える
pub fn on_scenario_text_changed(
    _trigger: On<ScenarioTextChanged>,
    state: Res<ScenarioState>,
    mut speaker_q: Single<&mut Text, (With<SpeakerText>, Without<MessageText>)>,
    mut message_q: Single<&mut Text, (With<MessageText>, Without<SpeakerText>)>,
) {
    let (speaker_str, message_str) = match state.current_step() {
        Some(Step::Dialogue { speaker, text, .. }) => (speaker.as_str(), text.as_str()),
        Some(Step::Narration { text }) => ("", text.as_str()),
        Some(Step::Command(_)) | None => ("", ""),
    };

    ***speaker_q = speaker_str.to_string();
    ***message_q = message_str.to_string();
}
