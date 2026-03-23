use crate::save::{
    LoadSlot, SaveLoadCommand, SaveLoadCommandQueue, SaveLoadUiState, SystemBarState,
};
use bevy::prelude::*;

const ANIM_SPEED: f32 = 6.0;
// ボタン 4 つ (70px) + gap 3 つ (6px) = 298px
const INNER_MAX_PX: f32 = 298.0;

// ────────────────────────────────────────────────
// Components
// ────────────────────────────────────────────────

#[derive(Component)]
pub struct SystemBarRoot;

#[derive(Component)]
pub struct SystemBarInner;

#[derive(Component)]
pub struct TabButton;

#[derive(Component)]
pub struct SaveButton;

#[derive(Component)]
pub struct LoadButton;

#[derive(Component)]
pub struct PlaceholderButton(pub usize);

#[derive(Component)]
pub struct ModalOverlay;

#[derive(Component)]
pub struct ModalYesButton(pub ModalAction);

#[derive(Component)]
pub struct ModalNoButton;

#[derive(Component)]
pub struct LoadSlotButton(pub LoadSlot);

#[derive(Clone)]
pub enum ModalAction {
    Save,
    Load(LoadSlot),
}

// ────────────────────────────────────────────────
// Spawn: システムバー
// ────────────────────────────────────────────────

pub fn spawn_system_bar(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Regular.ttf");

    commands
        .spawn((
            Name::new("SystemBar_Root"),
            SystemBarRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(8.0),
                right: Val::Px(8.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                overflow: Overflow::clip(),
                ..default()
            },
            ZIndex(100),
        ))
        .with_children(|root| {
            // ── 展開領域（スライドアニメーション） ──────────────
            root.spawn((
                Name::new("SystemBar_Inner"),
                SystemBarInner,
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::FlexEnd,
                    column_gap: Val::Px(6.0),
                    width: Val::Px(0.0),
                    overflow: Overflow::clip(),
                    ..default()
                },
                BackgroundColor(Color::NONE),
            ))
            .with_children(|inner| {
                spawn_icon_button(inner, &font, "Save", SaveButton);
                spawn_icon_button(inner, &font, "Load", LoadButton);
                spawn_icon_button(inner, &font, "XXXX", PlaceholderButton(0));
                spawn_icon_button(inner, &font, "XXXX", PlaceholderButton(1));
            });

            // ── Tab ボタン ──────────────────────────────────
            root.spawn((
                Name::new("Tab_Button"),
                TabButton,
                Button,
                Node {
                    width: Val::Px(58.0),
                    height: Val::Px(32.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    border: UiRect::all(Val::Px(1.5)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.15, 0.15, 0.45, 0.95)),
                BorderColor::from(Color::srgba(0.5, 0.5, 1.0, 0.85)),
            ))
            .with_children(|btn| {
                btn.spawn((
                    Text::new("[ Tab ]"),
                    TextFont {
                        font: FontSource::Handle(font.clone()),
                        font_size: bevy::text::FontSize::Px(13.0),
                        ..default()
                    },
                    TextColor(Color::srgba(0.85, 0.85, 1.0, 1.0)),
                ));
            });
        });
}

fn spawn_icon_button(
    parent: &mut ChildSpawnerCommands,
    font: &Handle<Font>,
    label: &str,
    marker: impl Component,
) {
    parent
        .spawn((
            Name::new(format!("Bar_{label}_Btn")),
            marker,
            Button,
            Node {
                width: Val::Px(70.0),
                height: Val::Px(32.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.12, 0.12, 0.40, 0.95)),
            BorderColor::from(Color::srgba(0.4, 0.4, 0.9, 0.85)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: bevy::text::FontSize::Px(14.0),
                    ..default()
                },
                TextColor(Color::srgba(0.85, 0.85, 1.0, 1.0)),
            ));
        });
}

// ────────────────────────────────────────────────
// Spawn: モーダル共通ヘルパー
// ────────────────────────────────────────────────

fn spawn_modal_overlay(commands: &mut Commands) -> Entity {
    commands
        .spawn((
            Name::new("Modal_Overlay"),
            ModalOverlay,
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            ZIndex(200),
        ))
        .id()
}

/// モーダルボタンをスポーンし Entity を返す
fn spawn_modal_btn(
    parent: &mut ChildSpawnerCommands,
    label: &str,
    font: &Handle<Font>,
    bg: Color,
) -> Entity {
    parent
        .spawn((
            Button,
            Node {
                min_width: Val::Px(110.0),
                height: Val::Px(44.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(16.0), Val::Px(0.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                border: UiRect::all(Val::Px(1.5)),
                ..default()
            },
            BackgroundColor(bg),
            BorderColor::from(Color::srgba(0.6, 0.6, 1.0, 0.7)),
        ))
        .with_children(|btn| {
            btn.spawn((
                Text::new(label.to_string()),
                TextFont {
                    font: FontSource::Handle(font.clone()),
                    font_size: bevy::text::FontSize::Px(18.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        })
        .id()
}

fn modal_window_bundle() -> impl Bundle {
    (
        Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(40.0), Val::Px(30.0)),
            row_gap: Val::Px(20.0),
            border_radius: BorderRadius::all(Val::Px(10.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.22, 0.97)),
        BorderColor::from(Color::srgba(0.5, 0.5, 1.0, 0.8)),
    )
}

fn modal_msg(msg: &str, font: &Handle<Font>) -> impl Bundle {
    (
        Text::new(msg.to_string()),
        TextFont {
            font: FontSource::Handle(font.clone()),
            font_size: bevy::text::FontSize::Px(22.0),
            ..default()
        },
        TextColor(Color::WHITE),
        TextLayout::new_with_justify(Justify::Center),
    )
}

fn modal_row() -> impl Bundle {
    Node {
        flex_direction: FlexDirection::Row,
        column_gap: Val::Px(16.0),
        ..default()
    }
}

// ────────────────────────────────────────────────
// Spawn: 各モーダル
// ────────────────────────────────────────────────

pub fn spawn_save_modal(commands: &mut Commands, asset_server: &Res<AssetServer>) {
    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Regular.ttf");
    let overlay = spawn_modal_overlay(commands);
    commands.entity(overlay).with_children(|ov| {
        ov.spawn(modal_window_bundle()).with_children(|win| {
            win.spawn(modal_msg("セーブしますか？", &font));
            win.spawn(modal_row()).with_children(|row| {
                let yes = spawn_modal_btn(row, "YES", &font, Color::srgba(0.15, 0.40, 0.15, 0.95));
                row.commands()
                    .entity(yes)
                    .insert(ModalYesButton(ModalAction::Save));

                let no = spawn_modal_btn(row, "NO", &font, Color::srgba(0.40, 0.15, 0.15, 0.95));
                row.commands().entity(no).insert(ModalNoButton);
            });
        });
    });
}

pub fn spawn_load_choose_modal(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    manual_exists: bool,
    auto_exists: bool,
) {
    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Regular.ttf");
    let overlay = spawn_modal_overlay(commands);
    commands.entity(overlay).with_children(|ov| {
        ov.spawn(modal_window_bundle()).with_children(|win| {
            win.spawn(modal_msg("ロードするデータを選んでください", &font));

            win.spawn(modal_row()).with_children(|row| {
                let manual_label = if manual_exists {
                    "手動セーブ"
                } else {
                    "手動セーブ\n(なし)"
                };
                let manual_bg = if manual_exists {
                    Color::srgba(0.15, 0.30, 0.50, 0.95)
                } else {
                    Color::srgba(0.25, 0.25, 0.25, 0.70)
                };
                let manual_e = spawn_modal_btn(row, manual_label, &font, manual_bg);
                if manual_exists {
                    row.commands()
                        .entity(manual_e)
                        .insert(LoadSlotButton(LoadSlot::Manual));
                }

                let auto_label = if auto_exists {
                    "オートセーブ"
                } else {
                    "オートセーブ\n(なし)"
                };
                let auto_bg = if auto_exists {
                    Color::srgba(0.15, 0.30, 0.50, 0.95)
                } else {
                    Color::srgba(0.25, 0.25, 0.25, 0.70)
                };
                let auto_e = spawn_modal_btn(row, auto_label, &font, auto_bg);
                if auto_exists {
                    row.commands()
                        .entity(auto_e)
                        .insert(LoadSlotButton(LoadSlot::Auto));
                }
            });

            win.spawn(modal_row()).with_children(|row| {
                let cancel = spawn_modal_btn(
                    row,
                    "キャンセル",
                    &font,
                    Color::srgba(0.40, 0.15, 0.15, 0.95),
                );
                row.commands().entity(cancel).insert(ModalNoButton);
            });
        });
    });
}

pub fn spawn_load_confirm_modal(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    slot: LoadSlot,
) {
    let font: Handle<Font> = asset_server.load("fonts/NotoSansJP-Regular.ttf");
    let overlay = spawn_modal_overlay(commands);
    let label = match &slot {
        LoadSlot::Manual => "手動セーブデータ",
        LoadSlot::Auto => "オートセーブデータ",
    };
    let msg = format!("{label}\nをロードしますか？");
    commands.entity(overlay).with_children(|ov| {
        ov.spawn(modal_window_bundle()).with_children(|win| {
            win.spawn(modal_msg(&msg, &font));
            win.spawn(modal_row()).with_children(|row| {
                let yes = spawn_modal_btn(row, "YES", &font, Color::srgba(0.15, 0.40, 0.15, 0.95));
                row.commands()
                    .entity(yes)
                    .insert(ModalYesButton(ModalAction::Load(slot)));

                let no = spawn_modal_btn(row, "NO", &font, Color::srgba(0.40, 0.15, 0.15, 0.95));
                row.commands().entity(no).insert(ModalNoButton);
            });
        });
    });
}

// ────────────────────────────────────────────────
// Systems
// ────────────────────────────────────────────────

/// Tab ボタン or Tab キー → バー展開トグル
pub fn handle_tab_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<TabButton>)>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut bar_state: ResMut<SystemBarState>,
) {
    let tab_clicked = interaction_q.iter().any(|i| *i == Interaction::Pressed);
    let tab_key = keyboard.just_pressed(KeyCode::Tab);
    if tab_clicked || tab_key {
        bar_state.expanded = !bar_state.expanded;
    }
}

/// ease-out quad アニメーションで Inner 幅を補間
pub fn animate_system_bar(
    time: Res<Time>,
    mut bar_state: ResMut<SystemBarState>,
    mut inner_q: Query<&mut Node, With<SystemBarInner>>,
) {
    let target = if bar_state.expanded { 1.0_f32 } else { 0.0 };
    bar_state.anim_t = bar_state
        .anim_t
        .lerp(target, (ANIM_SPEED * time.delta_secs()).min(1.0));

    let eased = {
        let t = bar_state.anim_t;
        1.0 - (1.0 - t) * (1.0 - t)
    };

    for mut node in inner_q.iter_mut() {
        node.width = Val::Px(INNER_MAX_PX * eased);
    }
}

/// Save ボタン → ConfirmSave モーダルを開く
pub fn handle_save_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<SaveButton>)>,
    mut ui_state: ResMut<SaveLoadUiState>,
    bar_state: Res<SystemBarState>,
) {
    if !bar_state.expanded {
        return;
    }
    if interaction_q.iter().any(|i| *i == Interaction::Pressed) {
        *ui_state = SaveLoadUiState::ConfirmSave;
    }
}

/// Load ボタン → ChooseLoad モーダルを開く
pub fn handle_load_button(
    interaction_q: Query<&Interaction, (Changed<Interaction>, With<LoadButton>)>,
    mut ui_state: ResMut<SaveLoadUiState>,
    bar_state: Res<SystemBarState>,
) {
    if !bar_state.expanded {
        return;
    }
    if interaction_q.iter().any(|i| *i == Interaction::Pressed) {
        *ui_state = SaveLoadUiState::ChooseLoad;
    }
}

/// SaveLoadUiState が変化したらモーダルを再生成する
pub fn sync_modal(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    ui_state: Res<SaveLoadUiState>,
    modal_q: Query<Entity, With<ModalOverlay>>,
) {
    if !ui_state.is_changed() {
        return;
    }

    for e in modal_q.iter() {
        commands.entity(e).despawn_related::<Children>();
        commands.entity(e).despawn();
    }

    match ui_state.as_ref() {
        SaveLoadUiState::Hidden => {}
        SaveLoadUiState::ConfirmSave => {
            spawn_save_modal(&mut commands, &asset_server);
        }
        SaveLoadUiState::ChooseLoad => {
            let manual = crate::save::manual_save_exists();
            let auto = crate::save::auto_save_exists();
            spawn_load_choose_modal(&mut commands, &asset_server, manual, auto);
        }
        SaveLoadUiState::ConfirmLoad(slot) => {
            spawn_load_confirm_modal(&mut commands, &asset_server, slot.clone());
        }
    }
}

/// モーダル内ボタンの入力処理
pub fn handle_modal_buttons(
    yes_q: Query<(&Interaction, &ModalYesButton), Changed<Interaction>>,
    slot_q: Query<(&Interaction, &LoadSlotButton), Changed<Interaction>>,
    no_q: Query<&Interaction, (Changed<Interaction>, With<ModalNoButton>)>,
    mut ui_state: ResMut<SaveLoadUiState>,
    mut cmd_queue: ResMut<SaveLoadCommandQueue>,
) {
    for (interaction, yes_btn) in yes_q.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }
        match &yes_btn.0 {
            ModalAction::Save => {
                cmd_queue.0.push(SaveLoadCommand::DoSave);
                *ui_state = SaveLoadUiState::Hidden;
            }
            ModalAction::Load(slot) => {
                cmd_queue.0.push(SaveLoadCommand::DoLoad(slot.clone()));
                *ui_state = SaveLoadUiState::Hidden;
            }
        }
    }

    for (interaction, slot_btn) in slot_q.iter() {
        if *interaction == Interaction::Pressed {
            *ui_state = SaveLoadUiState::ConfirmLoad(slot_btn.0.clone());
        }
    }

    for interaction in no_q.iter() {
        if *interaction == Interaction::Pressed {
            *ui_state = match ui_state.as_ref() {
                SaveLoadUiState::ConfirmLoad(_) => SaveLoadUiState::ChooseLoad,
                _ => SaveLoadUiState::Hidden,
            };
        }
    }
}

/// ステップが進むたびにオートセーブを実行する
pub fn auto_save_on_advance(
    scenario_state: Res<crate::scenario::ScenarioState>,
    mut cmd_queue: ResMut<SaveLoadCommandQueue>,
    mut prev_step: Local<usize>,
) {
    if scenario_state.scene.is_none() {
        return;
    }
    let current = scenario_state.current_step;
    if current != *prev_step {
        *prev_step = current;
        cmd_queue.0.push(SaveLoadCommand::AutoSave);
    }
}
