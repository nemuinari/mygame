# mygame

Bevy を使ったビジュアルノベル / ADV プロトタイプ。

---

## 環境

| 項目         | 内容                    |
| ------------ | ----------------------- |
| Bevy         | 0.19（git main）        |
| Rust edition | 2024                    |
| ターゲット   | Windows / macOS / Linux |

---

## 設計方針

- **設定値は基本パーセント表記**（`Val::Percent`）で解像度非依存にする
- **`mod.rs` で各パーツを統括する**。プラグイン登録・システム登録はすべて `mod.rs` に集約する
- **各パーツごとに役割を分担する**。UI・シナリオ・入力など関心事を明確に分離する
- **シナリオデータはバイナリ外部管理**。RON ファイルとして `assets/scenarios/` に置き、再ビルドなしで差し替え可能にする
- **仮置きリソースはカラーブロックで代替**。立ち絵・背景は後から画像に差し替えるだけで OK な構造にする
- **Bevy の不安定 API には依存しない**。git main を使う都合上、リリース間で廃止・移動された API は使わず、安定した基本機能だけで設計する（詳細は後述）

---

## ディレクトリ構成

```
mygame_sample1
├── assets
│   ├── fonts
│   │   └── NotoSansJP-Regular.ttf
│   └── scenarios
│       └── chapter_01
│           └── scene_01.ron
├── src
│   ├── scenario
│   │   ├── mod.rs          # ScenarioPlugin / ScenarioState / キュー Resource 定義
│   │   ├── loader.rs       # RON ファイルの読み込み
│   │   └── types.rs        # Scene / Step / SceneCommand 型定義
│   ├── ui
│   │   ├── mod.rs          # UiModulePlugin / 全システム登録
│   │   ├── messenger.rs    # メッセージウィンドウ・タイプライター・選択肢 UI
│   │   ├── background.rs   # 背景レイヤー・ChangeBackground 処理
│   │   └── character.rs    # 立ち絵スロット（Left / Center / Right）
│   └── main.rs             # App 組み立て・プラグイン登録
├── Cargo.toml
└── README.md
```

---

## 実装済み機能

### タイプライター演出

- `TypewriterState` リソースで文字送り状態を管理
- 1文字あたり `0.04秒`（`TYPEWRITER_CHAR_INTERVAL` 定数で調整可能）
- **スキップ対応**：演出中にスペース / 左クリックで全文を即時表示
- 次のステップへの進行は演出完了後のみ受け付ける

### 選択肢システム

`Step::Choice` バリアント追加。

```ron
Choice(
    prompt: "何と答える？",
    options: [
        ChoiceOption( label: "「はい」", jump_to: None ),
        ChoiceOption( label: "「いいえ」", jump_to: None ),
    ],
),
```

- 選択肢が表示されている間は Space / クリックによる強制進行をブロック
- ホバー時・クリック時の色変化でフィードバック
- `jump_to` フィールドは将来のシーン分岐用（現在は次の step へ進む）

### 立ち絵表示（カラーブロック仮置き）

- Left / Center / Right の3スロット固定
- キャラクター ID ごとに異なる色のブロックを表示（差し替え構造）
- `ShowCharacter` / `HideCharacter` コマンドで制御

```ron
Command(ShowCharacter( id: "heroine_a", position: Left, expression: "normal" )),
Command(HideCharacter( position: Left )),
```

**将来の差し替え方法：** `character.rs` の `spawn_character_slots` 内プレースホルダーを `ImageNode` に置き換えるだけ。

### 背景切り替え（カラーブロック仮置き）

- `ChangeBackground(key)` コマンドで背景色を変更
- キーと色のマッピングは `background.rs` の `bg_color_for_key` 関数で管理

```ron
Command(ChangeBackground("classroom")),
```

**将来の差し替え方法：** `background.rs` の `handle_background_change` 内で `BackgroundColor` を `ImageNode` への切り替えに変えるだけ。

---

## モジュール仕様

### `src/scenario`

#### `types.rs` — データ型

```
Scene
└── steps: Vec<Step>
      ├── Dialogue { speaker, text, voice? }
      ├── Narration { text }
      ├── Choice { prompt?, options: Vec<ChoiceOption> }
      └── Command(SceneCommand)
            ├── ChangeBackground(String)
            ├── ShowCharacter { id, position, expression }
            ├── HideCharacter { position }
            ├── PlayBgm(String)
            ├── StopBgm
            └── WaitInput
```

#### `mod.rs` — モジュール間通信の仕組み

モジュール間の通知は 2 種類の仕組みを使い分けている。

**① Observer イベント（`commands.trigger`）**

`ScenarioTextChanged` のみ。テキスト・状態変化を Observer で受け取り、UI 側が即座に反応する用途に使う。Observer 内では `ResMut` しか安全に扱えないため、イベント発行は行わない。

**② 自作 Vec キュー（`Resource<Vec<T>>`）**

シーンコマンド（背景・立ち絵）と選択肢確定の通知に使う。Observer が `PendingSceneCommand` Resource に書き込み、次フレームの通常システム `flush_pending_command` がキューに積む。UI 側のシステムが `drain()` で消費する。

```
ScenarioTextChanged (trigger)
  └─► on_scenario_text_changed (Observer / UI)
  └─► dispatch_commands (Observer / Scenario)
        └─► PendingSceneCommand (Resource)
              └─► flush_pending_command (System)
                    └─► BackgroundChangeQueue / CharacterShowQueue / CharacterHideQueue
                          └─► handle_background_change / handle_character_show / handle_character_hide

ChoiceButton (UI Interaction)
  └─► handle_choice_buttons (System)
        └─► ChoiceSelectedQueue (Resource)
              └─► handle_choice_selected (System)
                    └─► commands.trigger(ScenarioTextChanged)
```

| キュー Resource         | 書き込み元              | 読み取り元                 |
| ----------------------- | ----------------------- | -------------------------- |
| `BackgroundChangeQueue` | `flush_pending_command` | `handle_background_change` |
| `CharacterShowQueue`    | `flush_pending_command` | `handle_character_show`    |
| `CharacterHideQueue`    | `flush_pending_command` | `handle_character_hide`    |
| `ChoiceSelectedQueue`   | `handle_choice_buttons` | `handle_choice_selected`   |

### `src/ui`

#### `messenger.rs`

| 要素                       | 説明                                         |
| -------------------------- | -------------------------------------------- |
| `TypewriterState`          | Resource。文字送り状態・スキップ処理を管理   |
| `update_typewriter`        | フレームごとに文字を1文字ずつ追加            |
| `on_scenario_text_changed` | Observer。タイプライター開始・選択肢 UI 更新 |
| `handle_choice_buttons`    | 選択肢ボタンのホバー・クリック処理           |

#### `background.rs`

- `BackgroundLayer` コンポーネントを持つ全画面ノード（`ZIndex(-1)`）
- `handle_background_change` で `BackgroundChangeQueue` を drain して背景色を更新

#### `character.rs`

- `CharacterSlot { position }` コンポーネントで3スロットを管理
- `handle_character_show` / `handle_character_hide` でキューを drain して表示切り替え

---

## シナリオ RON フォーマット

```ron
Scene(
    id: "chapter_id_scene_id",
    steps: [
        Command(ChangeBackground("classroom")),
        Command(ShowCharacter( id: "heroine_a", position: Left, expression: "normal" )),
        Narration( text: "ナレーションテキスト。" ),
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

---

## Bevy 0.19 git main との互換性メモ

Bevy は git main を使用しているため、安定版と異なる API 破壊が起きることがある。
このプロジェクトで実際に踏んだ問題と対処を記録する。

### ① `EventReader` / `EventWriter` / `add_event` の廃止

**問題**

`bevy::prelude::*` にも `bevy::ecs::event` にも `EventReader` / `EventWriter` が存在せず、
`App::add_event::<T>()` メソッドも存在しない。

```
error[E0432]: unresolved import `bevy::ecs::event::EventReader`
error[E0599]: no method named `add_event` found for mutable reference `&mut App`
```

**原因**

Bevy 0.19 git main でこれらの型・メソッドが廃止または移動された。

**対処**

Bevy 組み込みのイベントシステムを使わず、**`Resource` に `Vec<T>` を持つ自作キュー**方式に統一した。

```rust
// 定義
#[derive(Resource, Default)]
pub struct BackgroundChangeQueue(pub Vec<BackgroundChangeRequested>);

// 書き込み
bg_queue.0.push(BackgroundChangeRequested { key });

// 読み出し（消費）
for ev in queue.0.drain(..) { ... }
```

この方式は Bevy のバージョン変化に関係なく動作する。

### ② Observer 内で `EventWriter` / `ResMut<Events<T>>` が使えない

**問題**

Observer のシステムパラメータに `EventWriter<T>` や `ResMut<Events<T>>` を渡すとコンパイルエラーになる。

**原因**

Bevy の Observer は通常のシステムと異なる実行コンテキストを持つため、書き込み系の Event 操作が制限される。

**対処**

Observer は `ResMut<PendingSceneCommand>`（単純な Resource）への書き込みのみ行い、
実際のキュー書き込みは次フレームで実行される通常システム `flush_pending_command` に委譲する2段階方式を採用した。

```
Observer → PendingSceneCommand (Resource) → flush_pending_command (System) → Vec キュー
```

### ③ `TextFont::font` の型が `Handle<Font>` から `FontSource` に変更

**問題**

```
error[E0308]: mismatched types
  expected enum `FontSource`, found enum `Handle<_>`
```

**対処**

```rust
// 変更前
TextFont { font: font_handle.clone(), .. }

// 変更後
TextFont { font: FontSource::Handle(font_handle.clone()), .. }
```

### ④ `BorderRadius` がコンポーネントから `Node` のフィールドに移動

**問題**

```
error[E0277]: `(Name, ..., BorderRadius)` is not a `Bundle`
```

**対処**

```rust
// 変更前（コンポーネントとして追加）
.spawn(( ..., BorderRadius::all(Val::Px(6.0)) ))

// 変更後（Node のフィールドとして設定）
Node {
    border_radius: BorderRadius::all(Val::Px(6.0)),
    ..default()
}
```

### ⑤ RON 0.8 では `Option` フィールドに `Some(...)` が必須

**問題**

`Option<String>` フィールドに文字列をそのまま書くとパースエラーになる。

```
40:21: Expected option
```

**原因**

RON 0.8 は `Option<T>` の値を `Some(...)` / `None` と明示する必要がある。
JSON や TOML のように値をそのまま書くことはできない。

**対処**

```ron
// NG
prompt: "何と答える？",

// OK
prompt: Some("何と答える？"),
prompt: None,
```

`#[serde(default)]` を付けたフィールドはRONファイルに書かなければ `None` になるため、
省略したい場合はフィールドごと書かないのが最もシンプル。

### ⑥ `Children::iter()` の戻り値が `&Entity` から `Entity` に変更

**問題**

```
error[E0308]: expected `Entity`, found `&_`
```

**対処**

```rust
// 変更前
for &child in children.iter() { ... }

// 変更後
for child in children.iter() { ... }
```

---

## 今後の予定

- セーブ＆ロード — `ScenarioState`（シーン ID・step インデックス）をファイルに永続化
- システムボタン — Save / Load / Conf ボタンを画面上部に配置
- 立ち絵の画像差し替え — `character.rs` のプレースホルダーを `ImageNode` に変更
- 背景画像の差し替え — `background.rs` の `BackgroundColor` を `ImageNode` に変更
- `jump_to` によるシーン分岐 — 選択肢でシーンファイルをロードし直す
- タイプライター音声 — 文字送りと同期した SE 再生
- オート送り / スキップ — 一定時間で自動進行・長押し高速送り
