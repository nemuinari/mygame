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
    /// 演出コマンド（将来実装）
    #[allow(dead_code)]
    Command(SceneCommand),
}

/// 演出コマンドの種類（将来実装）
#[allow(dead_code)]
#[derive(Debug, Deserialize, Clone)]
pub enum SceneCommand {
    ChangeBackground(String),
    PlayBgm(String),
    StopBgm,
    WaitInput,
}
