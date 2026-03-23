# mygame

Bevy (0.19 git main) を使ったビジュアルノベル / ADV プロトタイプ。

---

## 環境

| 項目         | 内容                    |
| ------------ | ----------------------- |
| Bevy         | 0.19（git main）        |
| Rust edition | 2024                    |
| ターゲット   | Windows / macOS / Linux |

---

## ディレクトリ構成

```
mygame
├── assets/
│   ├── fonts/NotoSansJP-Regular.ttf
│   └── scenarios/chapter_01/scene_01.ron, scene_02.ron
├── saves/
│   ├── manual.json     # 手動セーブ
│   └── auto.json       # オートセーブ
└── src/
    ├── main.rs                    # App 組み立て・プラグイン登録
    ├── save/mod.rs                # SavePlugin・セーブ/ロード処理・SaveLoadCommandQueue
    ├── scenario/
    │   ├── mod.rs                 # ScenarioPlugin・ScenarioState・キュー Resource
    │   ├── loader.rs              # RON 読み込み
    │   └── types.rs               # Scene / Step / SceneCommand 型定義
    └── ui/
        ├── mod.rs                 # UiModulePlugin・全システム登録
        ├── messenger.rs           # メッセージウィンドウ・タイプライター・選択肢 UI
        ├── background.rs          # 背景レイヤー
        ├── character.rs           # 立ち絵スロット (Left / Center / Right)
        └── system_buttons.rs      # システムバー・Save/Load モーダル
```

---

## キー操作

| キー               | 動作                                |
| ------------------ | ----------------------------------- |
| Space / 左クリック | テキスト進行 / タイプライタースキップ |
| Tab                | システムバー展開 / 閉じる           |
| 1〜9               | 選択肢を番号で選択                  |

---

## アーキテクチャ

### モジュール間通信

イベントシステムは廃止済みのため、**① Observer trigger** と **② 自作 Vec キュー** の2方式を使用。

```
ScenarioTextChanged (trigger)
  ├─► on_scenario_text_changed    # UI 更新 (messenger.rs)
  └─► dispatch_commands           # → PendingSceneCommand → flush_pending_command
                                  #   → BackgroundChangeQueue / CharacterShowQueue / CharacterHideQueue

ChoiceButton (Interaction)
  └─► handle_choice_buttons → ChoiceSelectedQueue → handle_choice_selected
        └─► trigger(ScenarioTextChanged)

SaveButton / LoadButton (Interaction)
  └─► SaveLoadUiState → sync_modal → handle_modal_buttons
        └─► SaveLoadCommandQueue → process_save_load_commands
```

| キュー                  | 書き込み元              | 読み取り元                   |
| ----------------------- | ----------------------- | ----------------------------- |
| `BackgroundChangeQueue` | `flush_pending_command` | `handle_background_change`    |
| `CharacterShowQueue`    | `flush_pending_command` | `handle_character_show`       |
| `CharacterHideQueue`    | `flush_pending_command` | `handle_character_hide`       |
| `ChoiceSelectedQueue`   | `handle_choice_buttons` | `handle_choice_selected`      |
| `SaveLoadCommandQueue`  | `handle_modal_buttons`  | `process_save_load_commands`  |

### モジュール一覧

| ファイル               | 主な責務                                                  |
| ---------------------- | --------------------------------------------------------- |
| `scenario/types.rs`    | `Scene` / `Step` / `SceneCommand` / `ChoiceOption` 型定義 |
| `scenario/loader.rs`   | RON ファイル → `Scene` パース                            |
| `scenario/mod.rs`      | `ScenarioState`・キュー・`advance_on_input`・Observer     |
| `save/mod.rs`          | JSON 永続化・`SaveLoadUiState`・`SaveLoadCommandQueue`     |
| `ui/messenger.rs`      | タイプライター (`TypewriterState`)・選択肢ボタン          |
| `ui/background.rs`     | 全画面背景ノード (`ZIndex(-1)`)・色切り替え               |
| `ui/character.rs`      | 立ち絵スロット × 3・カラーブロック仮置き                  |
| `ui/system_buttons.rs` | Tab 展開バー・Save/Load モーダル・オートセーブトリガー    |

---

## シナリオ RON フォーマット

```ron
Scene(
    id: "chapter_01_scene_01",
    steps: [
        Command(ChangeBackground("classroom")),
        Command(ShowCharacter( id: "heroine_a", position: Left, expression: "normal" )),
        Narration( text: "ナレーション。" ),
        Dialogue( speaker: "キャラ名", text: "セリフ。" ),
        Choice(
            prompt: Some("どうする？"),
            options: [
                ChoiceOption( label: "選択肢A", jump_to: None ),
                ChoiceOption( label: "選択肢B", jump_to: None ),
            ],
        ),
        Command(HideCharacter( position: Left )),
    ],
)
```

`scene_id` 命名規則: `chapter_XX_scene_YY` → `assets/scenarios/chapter_XX/scene_YY.ron`

---

## Bevy 0.19 互換メモ

| # | 問題 | 対処 |
| - | ---- | ---- |
| ① | `EventReader` / `EventWriter` / `add_event` 廃止 | `Resource<Vec<T>>` の自作キューで代替 |
| ② | Observer 内で `EventWriter` 不可 | `ResMut<PendingXxx>` Resource 経由で次フレームの通常システムに委譲 |
| ③ | `TextFont::font` が `Handle<Font>` → `FontSource` | `FontSource::Handle(handle)` でラップ |
| ④ | `BorderRadius` がコンポーネント → `Node` のフィールド | `Node { border_radius: ..., ..default() }` に移動 |
| ⑤ | RON 0.8 で `Option<T>` に `Some(...)` が必須 | `prompt: Some("…")` / `prompt: None` と明示。省略したい場合はフィールドごと書かない |
| ⑥ | `Children::iter()` が `&Entity` → `Entity` | `for child in children.iter()` （`&` 不要） |
| ⑦ | `JustifyText` → `Justify` に改名 | `Justify::Center` を使用 |
| ⑧ | UI ボタンクリックがシナリオ進行に干渉 | `button_q.iter().any(\|i\| *i == Interaction::Pressed)` で進行をブロック |

---

## 今後の予定

- 立ち絵・背景を画像に差し替え（`ImageNode` 化）
- `jump_to` によるシーン分岐
- タイプライター SE 再生
- システムバー `[ XXXX ]` ボタンの機能実装（既読スキップ・設定など）
