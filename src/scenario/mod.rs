pub mod loader;
pub mod types;

use bevy::prelude::*;
use std::path::PathBuf;
use types::{Scene, SceneCommand, Step};

// ────────────────────────────────────────────────
// リソース
// ────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct ScenarioState {
    pub scene: Option<Scene>,
    pub current_step: usize,
    pub waiting_choice: bool,
}

impl ScenarioState {
    pub fn current_step(&self) -> Option<&Step> {
        self.scene
            .as_ref()
            .and_then(|s| s.steps.get(self.current_step))
    }

    /// 選択肢以外のステップで次へ進む
    pub fn advance(&mut self) -> bool {
        if self.waiting_choice {
            return false;
        }
        if let Some(scene) = &self.scene {
            if self.current_step + 1 < scene.steps.len() {
                self.current_step += 1;
                return true;
            }
        }
        false
    }

    /// 選択肢を選んで次の step へ進む
    pub fn select_choice(&mut self, _index: usize) {
        self.waiting_choice = false;
        if let Some(scene) = &self.scene {
            if self.current_step + 1 < scene.steps.len() {
                self.current_step += 1;
            }
        }
    }

    /// 現在の Choice の選択肢数を返す（キー入力の範囲チェック用）
    pub fn choice_count(&self) -> usize {
        match self.current_step() {
            Some(Step::Choice { options, .. }) => options.len(),
            _ => 0,
        }
    }
}

// ────────────────────────────────────────────────
// 自作イベントキュー
// ────────────────────────────────────────────────

#[derive(Resource, Default, Clone)]
pub struct BackgroundChangeRequested {
    pub key: String,
}

#[derive(Resource, Default)]
pub struct BackgroundChangeQueue(pub Vec<BackgroundChangeRequested>);

#[derive(Clone)]
pub struct CharacterShowRequested {
    pub id: String,
    pub position: types::CharacterPosition,
    /// 表情キー（将来の差し替え用）
    #[allow(dead_code)]
    pub expression: String,
}

#[derive(Resource, Default)]
pub struct CharacterShowQueue(pub Vec<CharacterShowRequested>);

#[derive(Clone)]
pub struct CharacterHideRequested {
    pub position: types::CharacterPosition,
}

#[derive(Resource, Default)]
pub struct CharacterHideQueue(pub Vec<CharacterHideRequested>);

#[derive(Clone)]
pub struct ChoiceSelectedEvent {
    pub index: usize,
}

#[derive(Resource, Default)]
pub struct ChoiceSelectedQueue(pub Vec<ChoiceSelectedEvent>);

/// Observer → System 橋渡し：シーンコマンド
#[derive(Resource, Default)]
pub struct PendingSceneCommand(pub Option<SceneCommand>);

/// Observer → System 橋渡し：Choice ステップ到達フラグ
/// Observer 内では ResMut<ScenarioState> を取れないため、
/// フラグを Resource で渡して次フレームの通常システムでセットする
#[derive(Resource, Default)]
pub struct PendingWaitingChoice(pub bool);

// ────────────────────────────────────────────────
// Observer イベント
// ────────────────────────────────────────────────

#[derive(Event, Clone)]
pub struct ScenarioTextChanged;

// ────────────────────────────────────────────────
// Plugin
// ────────────────────────────────────────────────

pub struct ScenarioPlugin {
    pub initial_scene_path: PathBuf,
}

impl Plugin for ScenarioPlugin {
    fn build(&self, app: &mut App) {
        let path = self.initial_scene_path.clone();

        let state = match loader::load_scene(&path) {
            Ok(scene) => {
                info!("シーンをロードしました: {}", scene.id);
                ScenarioState {
                    scene: Some(scene),
                    current_step: 0,
                    waiting_choice: false,
                }
            }
            Err(e) => {
                warn!("シーンのロードに失敗しました: {e}");
                ScenarioState::default()
            }
        };

        app.insert_resource(state)
            .insert_resource(PendingSceneCommand::default())
            .insert_resource(PendingWaitingChoice::default())
            .insert_resource(BackgroundChangeQueue::default())
            .insert_resource(CharacterShowQueue::default())
            .insert_resource(CharacterHideQueue::default())
            .insert_resource(ChoiceSelectedQueue::default())
            .add_systems(Startup, emit_initial_text)
            .add_systems(Update, flush_pending_command)
            .add_observer(dispatch_commands);
    }
}

// ────────────────────────────────────────────────
// Systems
// ────────────────────────────────────────────────

fn emit_initial_text(mut commands: Commands) {
    commands.trigger(ScenarioTextChanged);
}

/// PendingWaitingChoice フラグを ScenarioState に反映する
/// Observer が直接 ResMut<ScenarioState> を取れないための中継システム
pub fn apply_pending_waiting_choice(
    mut pending: ResMut<PendingWaitingChoice>,
    mut state: ResMut<ScenarioState>,
) {
    if pending.0 {
        debug!("選択肢待ち状態に移行");
        state.waiting_choice = true;
        pending.0 = false;
    }
}

/// スペース・数字キー・左クリックの入力を処理する
///
/// 選択肢待ち中の動作：
///   [Space] または [1]  → 0 番目（1 番目）の選択肢を選択
///   [2]〜[9]           → 対応する選択肢を選択（範囲外は無視）
///   左クリック          → 無視（ChoiceButton の Interaction で処理）
///
/// 通常時の動作：
///   [Space] または 左クリック → タイプライタースキップ or 次ステップへ
pub fn advance_on_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<ScenarioState>,
    mut commands: Commands,
    mut choice_queue: ResMut<ChoiceSelectedQueue>,
    typewriter: Option<Res<crate::ui::messenger::TypewriterState>>,
) {
    // ── 選択肢待ち中 ──────────────────────────────
    if state.waiting_choice {
        // キーから選択肢インデックスを決定
        let key_index: Option<usize> = if keyboard.just_pressed(KeyCode::Space)
            || keyboard.just_pressed(KeyCode::Digit1)
            || keyboard.just_pressed(KeyCode::Numpad1)
        {
            Some(0)
        } else if keyboard.just_pressed(KeyCode::Digit2) || keyboard.just_pressed(KeyCode::Numpad2)
        {
            Some(1)
        } else if keyboard.just_pressed(KeyCode::Digit3) || keyboard.just_pressed(KeyCode::Numpad3)
        {
            Some(2)
        } else if keyboard.just_pressed(KeyCode::Digit4) || keyboard.just_pressed(KeyCode::Numpad4)
        {
            Some(3)
        } else if keyboard.just_pressed(KeyCode::Digit5) || keyboard.just_pressed(KeyCode::Numpad5)
        {
            Some(4)
        } else if keyboard.just_pressed(KeyCode::Digit6) || keyboard.just_pressed(KeyCode::Numpad6)
        {
            Some(5)
        } else if keyboard.just_pressed(KeyCode::Digit7) || keyboard.just_pressed(KeyCode::Numpad7)
        {
            Some(6)
        } else if keyboard.just_pressed(KeyCode::Digit8) || keyboard.just_pressed(KeyCode::Numpad8)
        {
            Some(7)
        } else if keyboard.just_pressed(KeyCode::Digit9) || keyboard.just_pressed(KeyCode::Numpad9)
        {
            Some(8)
        } else {
            None
        };

        if let Some(index) = key_index {
            let count = state.choice_count();
            if index < count {
                debug!("選択肢をキーで選択: index={}", index);
                state.waiting_choice = false;
                choice_queue.0.push(ChoiceSelectedEvent { index });
            }
        }
        // 選択肢待ち中はクリックによる誤進行を防ぐため、ここで必ず return
        return;
    }

    // ── 通常進行 ──────────────────────────────────
    let triggered = keyboard.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left);
    if !triggered {
        return;
    }

    // タイプライター演出中ならスキップ
    if let Some(tw) = typewriter {
        if !tw.finished {
            commands.trigger(ScenarioTextChanged);
            return;
        }
    }

    if state.advance() {
        commands.trigger(ScenarioTextChanged);
    }
}

pub fn handle_choice_selected(
    mut queue: ResMut<ChoiceSelectedQueue>,
    mut state: ResMut<ScenarioState>,
    mut commands: Commands,
) {
    let items: Vec<_> = queue.0.drain(..).collect();
    for ev in items {
        debug!("選択肢確定処理: index={}", ev.index);
        state.select_choice(ev.index);
        commands.trigger(ScenarioTextChanged);
    }
}

/// Observer: ScenarioTextChanged
///   - Command ステップ → PendingSceneCommand に書き込む
///   - Choice  ステップ → PendingWaitingChoice フラグを立てる
fn dispatch_commands(
    _trigger: On<ScenarioTextChanged>,
    state: Res<ScenarioState>,
    mut pending_cmd: ResMut<PendingSceneCommand>,
    mut pending_choice: ResMut<PendingWaitingChoice>,
) {
    match state.current_step() {
        Some(Step::Command(cmd)) => {
            pending_cmd.0 = Some(cmd.clone());
            pending_choice.0 = false;
        }
        Some(Step::Choice { .. }) => {
            pending_cmd.0 = None;
            pending_choice.0 = true;
        }
        _ => {
            pending_cmd.0 = None;
            pending_choice.0 = false;
        }
    }
}

fn flush_pending_command(
    mut pending: ResMut<PendingSceneCommand>,
    mut bg_queue: ResMut<BackgroundChangeQueue>,
    mut char_show_queue: ResMut<CharacterShowQueue>,
    mut char_hide_queue: ResMut<CharacterHideQueue>,
) {
    let Some(cmd) = pending.0.take() else { return };

    match cmd {
        SceneCommand::ChangeBackground(key) => {
            bg_queue.0.push(BackgroundChangeRequested { key });
        }
        SceneCommand::ShowCharacter {
            id,
            position,
            expression,
        } => {
            char_show_queue.0.push(CharacterShowRequested {
                id,
                position,
                expression,
            });
        }
        SceneCommand::HideCharacter { position } => {
            char_hide_queue.0.push(CharacterHideRequested { position });
        }
        SceneCommand::PlayBgm(_) | SceneCommand::StopBgm | SceneCommand::WaitInput => {}
    }
}
