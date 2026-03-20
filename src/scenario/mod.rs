pub mod loader;
pub mod types;

use bevy::prelude::*;
use std::path::PathBuf;
use types::{Scene, Step};

// ────────────────────────────────────────────────
// リソース：現在再生中のシーン状態
// ────────────────────────────────────────────────

/// シーンの進行状態をグローバルリソースとして保持する
#[derive(Resource, Default)]
pub struct ScenarioState {
    pub scene: Option<Scene>,
    /// 現在表示中の steps インデックス
    pub current_step: usize,
}

impl ScenarioState {
    /// 現在のステップを返す
    pub fn current_step(&self) -> Option<&Step> {
        self.scene
            .as_ref()
            .and_then(|s| s.steps.get(self.current_step))
    }

    /// 次のステップへ進む。終端なら false を返す
    pub fn advance(&mut self) -> bool {
        if let Some(scene) = &self.scene {
            if self.current_step + 1 < scene.steps.len() {
                self.current_step += 1;
                return true;
            }
        }
        false
    }
}

// ────────────────────────────────────────────────
// イベント：テキスト更新の通知
// ────────────────────────────────────────────────

/// シナリオのテキストが変化したことを UI 側に通知するイベント（Observer用）
/// Bevy 0.17+ では MessageWriter の代わりに commands.trigger() + Observer を推奨
#[derive(Event, Clone)]
pub struct ScenarioTextChanged;

// ────────────────────────────────────────────────
// Plugin
// ────────────────────────────────────────────────

pub struct ScenarioPlugin {
    /// 起動時に読み込むシーンファイルパス
    pub initial_scene_path: PathBuf,
}

impl Plugin for ScenarioPlugin {
    fn build(&self, app: &mut App) {
        let path = self.initial_scene_path.clone();

        // 初期シーンをロードして Resource に登録
        let state = match loader::load_scene(&path) {
            Ok(scene) => {
                info!("シーンをロードしました: {}", scene.id);
                ScenarioState {
                    scene: Some(scene),
                    current_step: 0,
                }
            }
            Err(e) => {
                warn!("シーンのロードに失敗しました: {e}");
                ScenarioState::default()
            }
        };

        app.insert_resource(state)
            .add_systems(Startup, emit_initial_text)
            .add_systems(Update, advance_on_input);
    }
}

// ────────────────────────────────────────────────
// Systems
// ────────────────────────────────────────────────

/// 起動直後に最初のテキストを UI へ通知する
fn emit_initial_text(mut commands: Commands) {
    commands.trigger(ScenarioTextChanged);
}

/// スペースキー or マウス左クリックで次のステップへ進む
fn advance_on_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut state: ResMut<ScenarioState>,
    mut commands: Commands,
) {
    let triggered = keyboard.just_pressed(KeyCode::Space) || mouse.just_pressed(MouseButton::Left);

    if triggered {
        state.advance();
        commands.trigger(ScenarioTextChanged);
    }
}
