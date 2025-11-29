# App / UI Requirements — Elm Architecture Layer

## 1️⃣ Scope & Purpose

- この文書は「アプリケーション/UI層」の要件を定義する。
- レンダリングコア(SceneContext)とCADコア(BRep等)の上に乗る **Elmアーキテクチャのホスト** としての役割に焦点を当てる。

## 2️⃣ Design Philosophy

- **Elm アーキテクチャ**
  - `Model / Msg / update / view` を基本とする。
  - `Model` はCADコアの状態+UI状態(ツールモード/選択/カメラ等)を含む。
  - `update` の中からのみ SceneContext や CAD コアの API を呼ぶ。

- **SceneContext はサービス**
  - App は SceneContext を直接いじらず、`update` 内の明示的な分岐からのみ操作する。
  - SceneContext の状態はUIの Model にはコピーせず、「描画・選択のためのサービス」として扱う。

## 3️⃣ Model / Msg 基本要件

- Model
  - `cad_state`: CADコアが管理するモデル(ドキュメント、ボディ、スケッチ等)へのハンドル。
  - `view_state`: カメラ、表示モード(ワイヤフレーム/シェーディング等)、選択中の要素ID。
  - `tool_state`: 現在のツール(選択/パン/回転/スケッチ/押し出しなど)の状態。

- Msg (例)
  - `MouseDown`, `MouseMove`, `MouseUp`, `KeyDown`, `KeyUp`
  - `ToolChanged`, `SelectionChanged`, `CameraChanged`
  - `CadOpRequested` (押し出し/ブーリアン等), `CadOpCompleted`

## 4️⃣ Integration with SceneContext

- update の中での責務分離:
  - 入力イベント→ Msg 化。
  - Msg に応じて Model を更新。
  - 必要に応じて SceneContext へ高レベルAPI呼び出し(表示/非表示、選択/ハイライト、Transform変更など)。

- SceneContext との境界:
  - SceneContext の状態(VisualFlags 等)を UI Model に重複保持しない。
  - レンダリング結果(スクリーンショット/フレーム)は UI から参照するが、SceneContext 内部を覗かない。

## 5️⃣ Integration with HTTP Command Server

- コマンドサーバはテスト/自動化/AIエージェント用のインターフェース。
- UI層は以下の2パターンを許容:
  1. App 自身が SceneContext を直接操作する通常モード。
  2. 外部エージェントが HTTP 経由で SceneContext を操作し、UIは結果のみを表示・観察するモード。
- どちらの場合も、SceneContext の public API が単一の真実の窓口であること。

## 6️⃣ Non-Functional

- 入力遅延: マウス操作→カメラ/選択更新→描画 までが人間の操作でストレスにならない応答性。
- 一貫性: UIはCADコアとレンダリングコアの状態を矛盾なく表示すること(例: 選択中のエッジが必ずハイライトされる)。
- テスト容易性: Msg シーケンスをテキストで記述し、SceneContext/HTTPを介して再生・検証できること。
 - ビュー操作: Pan/Zoom は `view_state.camera` の更新のみを行い、CADモデルや履歴(Undo/Redo対象)には影響しない。コマンド実行中(CommandActive)でも実行可能とする。

## 7️⃣ Out of Scope (for now)

- 本格的なUIフレームワーク選定(egui, winit, web UI 等)
- マルチドキュメント、プラグインシステム
