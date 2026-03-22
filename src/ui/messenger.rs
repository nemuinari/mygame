use crate::scenario::types::Step;
use crate::scenario::{
    ChoiceSelectedEvent, ChoiceSelectedQueue, ScenarioState, ScenarioTextChanged,
};
use bevy::prelude::*;
use bevy::text::FontSize;

const TYPEWRITER_CHAR_INTERVAL: f32 = 0.04;

// ────────────────────────────────────────────────
// Resource
// ────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct TypewriterState {
    pub full_text: String,
    pub visible_chars: usize,
    pub timer: f32,
    pub finished: bool,
    pub skip_requested: bool,
}

impl TypewriterState {
    pub fn start(&mut self, text: String) {
        self.full_text = text;
        self.visible_chars = 0;
        self.timer = 0.0;
        self.finished = false;
        self.skip_requested = false;
    }

    pub fn skip(&mut self) {
        self.skip_requested = true;
    }

    pub fn visible_text(&self) -> &str {
        let end = self
            .full_text
            .char_indices()
            .nth(self.visible_chars)
            .map(|(i, _)| i)
            .unwrap_or(self.full_text.len());
        &self.full_text[..end]
    }
}

// ────────────────────────────────────────────────
// Components
// ────────────────────────────────────────────────

#[derive(Component)]
pub struct MessageText;

#[derive(Component)]
pub struct SpeakerText;

#[derive(Component)]
pub struct ChoiceButton {
    pub index: usize,
}

#[derive(Component)]
pub struct ChoiceContainer;

/// ボタンの前フレームの Interaction を記録するコンポーネント
/// Pressed → Hovered/None の遷移で「クリック完了」を確実に検出する
#[derive(Component)]
pub struct PreviousInteraction(pub Interaction);

impl Default for PreviousInteraction {
    fn default() -> Self {
        Self(Interaction::None)
    }
}

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
            frame.spawn((
                SpeakerText,
                Name::new("Speaker_Text"),
                Text::new(""),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::srgba(0.8, 0.9, 1.0, 1.0)),
            ));
            frame.spawn((
                MessageText,
                Name::new("Message_Text"),
                Text::new(""),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: FontSize::Px(24.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });

    commands.spawn((
        Name::new("Choice_Container"),
        ChoiceContainer,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(60.0),
            left: Val::Percent(20.0),
            bottom: Val::Percent(28.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            ..default()
        },
        Visibility::Hidden,
    ));
}

// ────────────────────────────────────────────────
// Systems
// ────────────────────────────────────────────────

pub fn update_typewriter(
    time: Res<Time>,
    mut tw: ResMut<TypewriterState>,
    mut message_q: Single<&mut Text, With<MessageText>>,
) {
    if tw.finished {
        return;
    }

    if tw.skip_requested {
        tw.visible_chars = tw.full_text.chars().count();
        tw.finished = true;
        tw.skip_requested = false;
        **message_q = Text::new(tw.full_text.clone());
        return;
    }

    let total = tw.full_text.chars().count();
    if tw.visible_chars >= total {
        tw.finished = true;
        return;
    }

    tw.timer -= time.delta_secs();
    if tw.timer <= 0.0 {
        tw.visible_chars += 1;
        tw.timer = TYPEWRITER_CHAR_INTERVAL;
        **message_q = Text::new(tw.visible_text().to_string());
        if tw.visible_chars >= total {
            tw.finished = true;
        }
    }
}

pub fn on_scenario_text_changed(
    _trigger: On<ScenarioTextChanged>,
    state: Res<ScenarioState>,
    mut speaker_q: Single<&mut Text, (With<SpeakerText>, Without<MessageText>)>,
    mut message_q: Single<&mut Text, (With<MessageText>, Without<SpeakerText>)>,
    mut tw: ResMut<TypewriterState>,
    choice_container_q: Single<(Entity, &mut Visibility), With<ChoiceContainer>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // タイプライター演出中に再トリガー = スキップ
    if !tw.finished && tw.visible_chars > 0 {
        tw.skip();
        return;
    }

    let (container_entity, mut container_vis) = choice_container_q.into_inner();
    commands
        .entity(container_entity)
        .despawn_related::<Children>();
    *container_vis = Visibility::Hidden;

    match state.current_step() {
        Some(Step::Dialogue { speaker, text, .. }) => {
            **speaker_q = Text::new(speaker.clone());
            **message_q = Text::new("");
            tw.start(text.clone());
        }
        Some(Step::Narration { text }) => {
            **speaker_q = Text::new("");
            **message_q = Text::new("");
            tw.start(text.clone());
        }
        Some(Step::Choice { prompt, options }) => {
            let prompt_text = prompt.clone().unwrap_or_default();
            **speaker_q = Text::new("");
            **message_q = Text::new("");
            tw.start(prompt_text);

            *container_vis = Visibility::Visible;
            let font = asset_server.load("fonts/NotoSansJP-Regular.ttf");
            let opts = options.clone();

            commands.entity(container_entity).with_children(|parent| {
                for (i, option) in opts.iter().enumerate() {
                    // キーヒント: [1]〜[9]、スペースは [1] と同じ
                    let key_hint = format!("[{}] ", i + 1);
                    let label = format!("{}{}", key_hint, option.label);
                    parent
                        .spawn((
                            Name::new(format!("Choice_Button_{i}")),
                            ChoiceButton { index: i },
                            PreviousInteraction::default(),
                            Button,
                            Node {
                                width: Val::Percent(100.0),
                                padding: UiRect::axes(Val::Px(24.0), Val::Px(12.0)),
                                justify_content: JustifyContent::FlexStart,
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                border: UiRect::all(Val::Px(1.5)),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.15, 0.15, 0.45, 0.95)),
                            BorderColor::from(Color::srgba(0.5, 0.5, 1.0, 0.9)),
                        ))
                        .with_children(|btn| {
                            btn.spawn((
                                Text::new(label),
                                TextFont {
                                    font: FontSource::Handle(font.clone()),
                                    font_size: FontSize::Px(22.0),
                                    ..default()
                                },
                                TextColor(Color::WHITE),
                            ));
                        });
                }
            });
        }
        Some(Step::Command(_)) | None => {
            **speaker_q = Text::new("");
            **message_q = Text::new("");
            tw.start(String::new());
        }
    }
}

pub fn handle_choice_buttons(
    mut interaction_q: Query<(
        &Interaction,
        &mut PreviousInteraction,
        &mut BackgroundColor,
        &ChoiceButton,
    )>,
    mut choice_queue: ResMut<ChoiceSelectedQueue>,
    mut state: ResMut<ScenarioState>,
) {
    for (interaction, mut prev, mut bg, btn) in interaction_q.iter_mut() {
        // Pressed → Hovered/None への遷移 = クリック完了
        let released = prev.0 == Interaction::Pressed && *interaction != Interaction::Pressed;

        // 外観の更新
        match *interaction {
            Interaction::Pressed => {
                *bg = BackgroundColor(Color::srgba(0.35, 0.35, 0.75, 0.95));
            }
            Interaction::Hovered => {
                *bg = BackgroundColor(Color::srgba(0.25, 0.25, 0.65, 0.95));
            }
            Interaction::None => {
                *bg = BackgroundColor(Color::srgba(0.15, 0.15, 0.45, 0.95));
            }
        }

        // クリック完了時のみ選択処理
        if released {
            debug!("選択肢クリック完了: index={}", btn.index);
            state.waiting_choice = false;
            choice_queue
                .0
                .push(ChoiceSelectedEvent { index: btn.index });
        }

        // 今フレームの状態を保存
        prev.0 = *interaction;
    }
}
