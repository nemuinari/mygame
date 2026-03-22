use serde::Deserialize;

/// 一シーン全体
#[derive(Debug, Deserialize, Clone)]
pub struct Scene {
    pub id: String,
    pub steps: Vec<Step>,
}

/// シーン内の1ステップ
#[derive(Debug, Deserialize, Clone)]
pub enum Step {
    /// セリフ（話者名 + テキスト）
    Dialogue {
        speaker: String,
        text: String,
        /// 将来の音声対応用（未実装）
        #[serde(default)]
        #[allow(dead_code)]
        voice: Option<String>,
    },
    /// ナレーション（話者なし）
    Narration { text: String },
    /// 選択肢
    Choice {
        prompt: Option<String>,
        options: Vec<ChoiceOption>,
    },
    /// 演出コマンド
    Command(SceneCommand),
}

/// 選択肢の一項目
#[derive(Debug, Deserialize, Clone)]
pub struct ChoiceOption {
    pub label: String,
    /// 選択後にジャンプするシーン ID（None なら次の step へ）
    #[serde(default)]
    #[allow(dead_code)]
    pub jump_to: Option<String>,
}

/// 立ち絵の表示位置
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub enum CharacterPosition {
    Left,
    Center,
    Right,
}

/// 演出コマンドの種類
#[derive(Debug, Deserialize, Clone)]
#[allow(dead_code)]
pub enum SceneCommand {
    ChangeBackground(String),
    ShowCharacter {
        id: String,
        position: CharacterPosition,
        /// 表情キー（将来の差し替え用）
        #[serde(default = "default_expression")]
        expression: String,
    },
    HideCharacter {
        position: CharacterPosition,
    },
    PlayBgm(String),
    StopBgm,
    WaitInput,
}

fn default_expression() -> String {
    "normal".to_string()
}
