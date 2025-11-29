# 📚 Rust CAD Rendering Core — 設計ポータル

このサイトは、Rust + wgpu + ECS ベースの
**CAD レンダリング・操作コアの設計仕様** をまとめたものです。

> 実装よりも先に、「壊れない設計」と「AIが守れるルール」を固めるフェーズです。

---

## 🧭 全体構成（設計ドキュメント）

- 設計概要・責務分離 → [Architecture Overview](./design/architecture-overview.md)
- CADコア設計 → [CAD Architecture Overview](./design/cad-architecture-overview.md)
- コマンドと状態遷移 → [Command State Machines](./design/command-state-machines.md)
- SceneContext状態遷移 → [State Transitions](./design/state-transitions.md)
- HTTP API → [HTTP API Spec](./design/http-api.md)
- レイヤ/プロパティ → [Layers & Properties](./design/layers-properties.md)
- スナップ/拘束 → [Snapping & Constraints](./design/snapping-constraints.md)
- 永続化(JSON v0) → [Persistence Model](./design/persistence-model.md)
- テスト/E2E → [Test Plan](./design/test-plan.md)
- App/Msg/FSMフロー → [App Interactions](./design/app-interactions.md)
- 設計の背景と履歴 → [Design Rationale](./articles/design-rationale-architecture.md)

要件レベルの文書は `docs/requirements/` にまとまっています：

- [Rendering Core 要件](./requirements/rendering-core.md)
- [CAD Core 要件](./requirements/cad-core.md)
- [App/UI 要件](./requirements/app-ui.md)
- [CAD Command Specification](./requirements/CAD_COMMAND_SPECIFICATION.md)

---

## 🧊 設計フェーズの状態

- 設計フェーズ: **デザインフリーズ済み**
- セーブポイント: タグ `spec-v0`
- 設計保存ブランチ: `design-spec-freeze`

実装が迷走した場合は、この状態から再スタートできます。

---

## 🧱 アーキテクチャの軸（かんたん要約）

### 1. SceneContext を中心にした「誤用できない」API

- 外部から触れるのは SceneContext の public API のみ。
- `submit_shape / set_visibility / set_highlight / set_selected / set_transform / remove / render / pick / get_state`
- `render()` は内部で `sync_gpu()` を実行し、dirty 残存なら `debug_assert!`
- `pick()` は状態を変えない純粋クエリ（ハイライトは `set_highlight` で明示更新）

### 2. CADコアとレンダリングの完全分離

- CADコアは差分ログ + Undo/Redo を持つ論理モデル。
- SceneContext/Rendering は「結果に追従するビュー」。
- Undo/Redo は CAD 状態のみを巻き戻し、Rendering 側は dirty から再同期。

### 3. 状態遷移（FSM）で操作を明文化

- 線分コマンド例: `WaitFirst → WaitSecond → Commit → (Continuousなら WaitFirst)`。
- Cancel/Esc はいつでもプレビュー削除 → Idle。
- プレビュー用 EntityId と確定用 EntityId は分離。
- 全コマンドはグローバルFSM（Idle→CommandActive→Finish/Cancel→Idle）上に乗る。

### 4. レイヤ/プロパティとスナップ/拘束の責務分離

- 最終可視: `final_visible = layer.visible && element.visible`。
- v0 の locked layer は選択/ハイライト/編集不可。
- Snap/拘束は PointInput 前処理: `PointInputResult { raw, snapped, snap_type }` をコマンドに渡す。

### 5. JSON 永続化（v0）

- フラット構造 + ID参照。右手系、単位はメートル。
- `kind` = 意味的エンティティ種別（Line/Wall...）、`geometry.type` = 形状表現（Line2D など）。
- 永続IDは UUID 等の安定IDで、GPUインデックスとは無関係。

---

## 🧪 テストとE2Eシナリオ

- ユニットテスト / HTTP API テスト。
- シナリオ駆動E2E（例: 線 → Move → Trim → Undo/Redo → 再描画）。
- `state_assert` は部分一致、数値は v0 では厳密比較。
- `screenshot_assert` は v0 では PNG ピクセル完全一致（将来 SSIM 等へ拡張）。

詳しくは 👉 [Test Plan](./design/test-plan.md)

---

## 🤖 AI開発者・コントリビュータへ

AIエージェント向けのルールはリポジトリ直下の **`AGENT_RULES.md`** にまとめています。コードを触る前に必ず仕様とルールを確認してください。

---

## 👋 最後に

ここに掲載されているのは「完成アプリ」ではなく、**長期運用可能なCADコアの設計書**です。設計が真実のソースであり続けます。仕様を読んで、もし興味があれば一緒に実装フェーズを進めましょう。
