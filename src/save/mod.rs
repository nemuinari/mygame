use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ────────────────────────────────────────────────
// セーブデータ型
// ────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveData {
    pub scene_id: String,
    pub current_step: usize,
    pub timestamp: u64,
}

// ────────────────────────────────────────────────
// Resource
// ────────────────────────────────────────────────

/// セーブ・ロード UI の状態
#[derive(Resource, Default, Clone, PartialEq)]
pub enum SaveLoadUiState {
    #[default]
    Hidden,
    /// セーブ確認モーダル
    ConfirmSave,
    /// ロード選択モーダル（手動 / オート どちらかを選ぶ）
    ChooseLoad,
    /// ロード確認モーダル（選択したスロット）
    ConfirmLoad(LoadSlot),
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoadSlot {
    Manual,
    Auto,
}

/// システムボタンバー（Tab展開）の状態
#[derive(Resource, Default)]
pub struct SystemBarState {
    /// バーが展開されているか
    pub expanded: bool,
    /// アニメーション進捗 0.0 (閉) ～ 1.0 (開)
    pub anim_t: f32,
}

// ────────────────────────────────────────────────
// セーブ・ロード の実処理
// ────────────────────────────────────────────────

fn manual_save_path() -> PathBuf {
    PathBuf::from("saves/manual.json")
}

fn auto_save_path() -> PathBuf {
    PathBuf::from("saves/auto.json")
}

pub fn write_save(data: &SaveData, path: &PathBuf) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_string_pretty(data)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn read_save(path: &PathBuf) -> Option<SaveData> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

pub fn manual_save_exists() -> bool {
    manual_save_path().exists()
}

pub fn auto_save_exists() -> bool {
    auto_save_path().exists()
}

pub fn save_manual(data: SaveData) -> anyhow::Result<()> {
    write_save(&data, &manual_save_path())
}

pub fn save_auto(data: SaveData) -> anyhow::Result<()> {
    write_save(&data, &auto_save_path())
}

pub fn load_manual() -> Option<SaveData> {
    read_save(&manual_save_path())
}

pub fn load_auto() -> Option<SaveData> {
    read_save(&auto_save_path())
}

// ────────────────────────────────────────────────
// コマンドキュー（UI → ロジック）
// ────────────────────────────────────────────────

#[derive(Resource, Default)]
pub struct SaveLoadCommandQueue(pub Vec<SaveLoadCommand>);

#[derive(Clone, Debug)]
pub enum SaveLoadCommand {
    DoSave,
    DoLoad(LoadSlot),
    AutoSave,
}

// ────────────────────────────────────────────────
// Plugin
// ────────────────────────────────────────────────

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SaveLoadUiState::default())
            .insert_resource(SystemBarState::default())
            .insert_resource(SaveLoadCommandQueue::default())
            .add_systems(Update, process_save_load_commands);
    }
}

// ────────────────────────────────────────────────
// System: コマンドを実行する
// ────────────────────────────────────────────────

fn process_save_load_commands(
    mut queue: ResMut<SaveLoadCommandQueue>,
    mut scenario_state: ResMut<crate::scenario::ScenarioState>,
    mut commands: Commands,
) {
    let items: Vec<_> = queue.0.drain(..).collect();
    for cmd in items {
        match cmd {
            SaveLoadCommand::DoSave | SaveLoadCommand::AutoSave => {
                let is_auto = matches!(cmd, SaveLoadCommand::AutoSave);
                let Some(scene) = &scenario_state.scene else {
                    warn!("セーブ失敗: シーンが読み込まれていません");
                    continue;
                };
                let data = SaveData {
                    scene_id: scene.id.clone(),
                    current_step: scenario_state.current_step,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                };
                let result = if is_auto {
                    save_auto(data)
                } else {
                    save_manual(data)
                };
                match result {
                    Ok(_) => info!("{}セーブ完了", if is_auto { "オート" } else { "手動" }),
                    Err(e) => warn!("セーブ失敗: {e}"),
                }
            }
            SaveLoadCommand::DoLoad(slot) => {
                let save_data = match slot {
                    LoadSlot::Manual => load_manual(),
                    LoadSlot::Auto => load_auto(),
                };
                let Some(data) = save_data else {
                    warn!("ロード失敗: セーブデータが存在しません");
                    continue;
                };

                // scene_id から単純に chapter_01/scene_01 形式を推定
                let scene_path = derive_scene_path(&data.scene_id);
                match crate::scenario::loader::load_scene(&std::path::PathBuf::from(&scene_path)) {
                    Ok(scene) => {
                        let step = data.current_step.min(scene.steps.len().saturating_sub(1));
                        scenario_state.scene = Some(scene);
                        scenario_state.current_step = step;
                        scenario_state.waiting_choice = false;
                        commands.trigger(crate::scenario::ScenarioTextChanged);
                        info!("ロード完了: step={}", step);
                    }
                    Err(e) => {
                        warn!("ロード失敗: シーンファイルを読めませんでした ({scene_path}): {e}");
                    }
                }
            }
        }
    }
}

/// scene_id ("chapter_01_scene_01") からファイルパスを推定する
/// 規則: 最初の "_scene_" の前をディレクトリ、後をファイル名とする
/// フォールバック: assets/scenarios/<scene_id>.ron
fn derive_scene_path(scene_id: &str) -> String {
    // "chapter_01_scene_01" → chapter_01 / scene_01.ron
    if let Some(pos) = scene_id.find("_scene_") {
        let chapter = &scene_id[..pos];
        let scene = &scene_id[pos + 1..]; // "scene_01"
        return format!("assets/scenarios/{chapter}/{scene}.ron");
    }
    format!("assets/scenarios/{scene_id}.ron")
}
