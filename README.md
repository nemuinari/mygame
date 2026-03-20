# mygame

Bevy を使ったビジュアルノベル / ADV プロトタイプ。

---

## 環境

| 項目         | 内容                    |
| ------------ | ----------------------- |
| Bevy         | 0.18（git main）        |
| Rust edition | 2024                    |
| ターゲット   | Windows / macOS / Linux |

---

## 設計方針

- **設定値は基本パーセント表記**（`Val::Percent`）で解像度非依存にする
- **`mod.rs` で各パーツを統括する**。プラグイン登録・システム登録はすべて `mod.rs` に集約し、個別ファイルに散らさない
- **各パーツごとに役割を分担する**。UI・シナリオ・入力など関心事を明確に分離する
- **シナリオデータはバイナリ外部管理**。RON ファイルとして `assets/scenarios/` に置き、再ビルドなしで差し替え可能にする

---

## ディレクトリ構成

```
mygame_sample1
├── assets
│   ├── fonts
│   │   └── NotoSansJP-Regular.ttf
│   └── scenarios
│       ├── chapter_01
│       │   ├── scene_01.ron        # 会話シーン（実装済み）
│       │   └── scene_02.ron        # （未作成）
│       └── chapter_02
│           └── scene_01.ron        # （未作成）
├── src
│   ├── scenario
│   │   ├── mod.rs                  # ScenarioPlugin / ScenarioState / ScenarioTextChanged
│   │   ├── loader.rs               # RON ファイルの読み込み
│   │   └── types.rs                # Scene / Step / SceneCommand 型定義
│   ├── ui
│   │   ├── mod.rs                  # UiModulePlugin / カメラ・各パーツの spawn
│   │   ├── messenger.rs            # メッセージウィンドウ（話者名 + 本文）
│   │   └── background.rs           # 背景レイヤー
│   └── main.rs                     # App 組み立て・プラグイン登録
├── Cargo.toml
└── README.md
```

---

## モジュール仕様

### `src/scenario`

シナリオデータの読み込みと進行状態を管理する。UI とは `ScenarioTextChanged`（Observer イベント）経由で疎結合に連携する。

#### `types.rs` — データ型

```
Scene
└── steps: Vec<Step>
      ├── Dialogue { speaker, text, voice? }   # セリフ（話者名付き）
      ├── Narration { text }                    # ナレーション（話者なし）
      └── Command(SceneCommand)                 # 演出コマンド（将来実装）
            ├── ChangeBackground(String)
            ├── PlayBgm(String)
            ├── StopBgm
            └── WaitInput
```

- `voice` フィールドは将来の音声対応用。現時点では未使用（`#[allow(dead_code)]`）
- `SceneCommand` は現時点では未使用（`#[allow(dead_code)]`）

#### `loader.rs` — ファイル読み込み

- `load_scene(path: &Path) -> Result<Scene>` を提供
- `ron::from_str` でデシリアライズ。失敗時は `anyhow::Error` を返す

#### `mod.rs` — プラグイン・状態管理

| 要素                  | 説明                                                        |
| --------------------- | ----------------------------------------------------------- |
| `ScenarioState`       | `Resource`。現在のシーンと step インデックスを保持          |
| `ScenarioTextChanged` | `Event`。テキストが変化したことを Observer 経由で UI へ通知 |
| `ScenarioPlugin`      | 起動時に RON をロードし、入力処理システムを登録             |

**入力トリガー**：`Space` キー または マウス左クリックで次の step へ進み、`commands.trigger(ScenarioTextChanged)` を発火する。

---

### `src/ui`

画面描画を担当する。シナリオの詳細を知らず、`ScenarioState` と Observer イベントのみを参照する。

#### `background.rs`

- `BackgroundLayer` コンポーネントを持つ全画面ノード（`ZIndex(-1)`）
- 背景色：`Color::srgba(0.02, 0.02, 0.1, 1.0)`

#### `messenger.rs`

メッセージウィンドウ全体を管理するモジュール。

**レイアウト**

| 項目       | 値                                              |
| ---------- | ----------------------------------------------- |
| 位置       | 画面下部・中央寄せ（`left: 5%`, `bottom: 3%`）  |
| サイズ     | 幅 `90%`、高さ `22%`                            |
| パディング | 水平 `2.5%`、垂直 `2.0%`                        |
| 背景       | `srgba(0.05, 0.05, 0.15, 0.92)`（半透明ダーク） |
| 枠線       | `srgba(0.6, 0.6, 1.0, 0.8)`（淡い青紫）         |

**テキスト要素**

| コンポーネント | フォントサイズ | 色                          | 説明                         |
| -------------- | -------------- | --------------------------- | ---------------------------- |
| `SpeakerText`  | `18px`         | `srgba(0.8, 0.9, 1.0, 1.0)` | 話者名。Narration 時は空文字 |
| `MessageText`  | `24px`         | `WHITE`                     | 本文                         |

**未実装・今後の対応予定**

- [ ] テキストの左寄せ / 中寄せ切り替え
- [ ] 最大3行表示（オーバーフロー制御）
- [ ] 1文字ずつ表示するタイプライター演出
- [ ] オート送り（一定時間で自動進行）
- [ ] スキップ（長押しまたはキー入力で高速送り）

---

### `src/main.rs`

プラグインの登録順：

1. `DefaultPlugins`（ウィンドウタイトル設定含む）
2. `ScenarioPlugin`（Resource を先に確保する必要があるため UI より前）
3. `UiModulePlugin`

---

## シナリオ RON フォーマット

```ron
Scene(
    id: "chapter_id_scene_id",
    steps: [
        Command(ChangeBackground("背景キー")),
        Command(PlayBgm("BGMキー")),
        Narration(
            text: "ナレーションテキスト。",
        ),
        Dialogue(
            speaker: "キャラクター名",
            text: "セリフテキスト。",
        ),
        Command(WaitInput),
    ],
)
```

---

## 依存クレート

| クレート | 用途                                 |
| -------- | ------------------------------------ |
| `bevy`   | ゲームエンジン本体（git main）       |
| `serde`  | シリアライズ / デシリアライズ derive |
| `ron`    | RON フォーマットのパーサー           |
| `anyhow` | エラーハンドリング                   |

---

## 今後の予定

- セーブ＆ロード
  - 現在の `ScenarioState`（シーン ID・step インデックス）をファイルに永続化し、再開できるようにする

- システムボタン
  - 画面上部などに Save / Load / Prof（プロフィール）/ Conf（設定）ボタンを配置する

- シナリオ選択肢
  - `Step` に `Choice { options: Vec<ChoiceOption> }` バリアントを追加し、分岐シナリオを実現する

- 立ち絵表示・操作
  - キャラクター画像を画面中央付近に表示し、位置・表情・フリップなどを `SceneCommand` で制御する

- 背景切り替え
  - `Command(ChangeBackground(...))` を実際に処理し、`background.rs` の背景画像を差し替える
