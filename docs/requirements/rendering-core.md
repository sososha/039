# Rust WebGPU CAD Rendering Core — Requirements Specification

---

## 1️⃣ Purpose

このドキュメントは、Rust + WebGPU (wgpu) ベースで開発する **CADレンダリングコア**の要件を定義します。  
最終目的は：

> **長期的に高速で壊れにくく、誤用不能なAPIを備えたCAD向け描画基盤を構築すること。**

CADコア(幾何/トポロジ/履歴)の要件は `docs/requirements/cad-core.md`、
アプリ/UI層(Elmアーキテクチャ)の要件は `docs/requirements/app-ui.md` を参照。

---

## 2️⃣ Design Philosophy

- **未来志向**  
  OpenGL 前提の設計を捨て、WebGPU 世代へ最適化する。
- **構造で安全性を担保**  
  `README注意事項方式`ではなく、`型、visibility、API設計`でミスを排除する。
- **AI補完前提**  
  コードの大部分はAIが生成する。従って、曖昧な仕様や揺れるAPIは禁止。
- **Elm的統治**  
  アプリ全体は Model / Msg / update の同期フローを基本とし、SceneContext は update 内から呼ばれるレンダリング・インタラクションのサービスとして分離する。

---

## 3️⃣ Core Architectural Principles

| 項目 | 要件 |
|------|------|
| GPU API | wgpu (WebGPU準拠) |
| 状態管理 | ECS 風 (`SceneWorld` + `systems`) |
| 外部API | `SceneContext` が唯一のフロント |
| 同期制御 | Dirtyフラグによる差分更新 |
| 安全性 | 内部処理は private / pub(crate) で封じる |
| 役割分離 | アプリ統治は Elm型、レンダリング統治は SceneContext 型 |

---

## 4️⃣ Functional Requirements

### 4.1 Entity Representation

- CADカーネルのBRep/CSG/Sketchは抽象化し、`KernelShape` trait で受ける
- Geometry → Mesh (triangle list) への離散化は `tessellate()`で統一
- EntityId は安定ID（再生成しない）。外部に露出するのはコピー可能な値型のみ。
- メッシュ/法線/マテリアルは immutable とし、差し替え時にのみ新リソース生成。
- GPUバッファ生成・ライフサイクルは `SceneContext` 内部の allocator/system が単一責任で管理。

### 4.2 Interaction States

各 EntityId は以下の状態を持つ：

- Visible
- Selected
- Highlighted

これらは `VisualFlags` でビット管理する。

### 4.3 Dirty-Based Rendering Loop

App
→ SceneContext API call
→ World 変更
→ Dirtyフラグ更新
→ sync_gpu()
→ render()

Dirty分類と伝搬ルール：
- `DirtyFlags::Geometry` : tessellate → mesh → GPU buffer 再構築
- `DirtyFlags::Visual` : Visible/Selected/Highlighted → instance/uniform 更新のみ
- `DirtyFlags::Transform` : モデル行列/法線行列更新 → instance buffer 再書き込み
- sync時に処理すべき順序を定義（Geometry→Transform→Visual）。
SceneContextは `render()` 前に dirty 非空なら `debug_assert!` を発火。外部が手動で sync を呼ぶ必要はない。

### 4.4 SceneContext API 制約

- `SceneContext` は描画状態変更の唯一の public API。内部の systems / wgpu は pub(crate)/private。
- API は状態遷移を安全に誘導する形で提供（例: `set_visibility`, `set_highlight`, `submit_shape`）。
- API は非同期を隠蔽し、呼び出しはノンブロッキング（GPU submit は内部で batch）。
- パニック要件：不整合（未知の EntityId、必要リソース未生成など）は debug ビルドで panic。本番ビルドではエラーを返す Result ベース。

### 4.5 AIS 思想の継承

- AIS (OpenCascade Interactive Services) の機能コピーではなく思想を踏襲する。
- 表示状態変更は必ずコンテキスト経由。外部が低レベルリソースを直接触らない。
- 高レベルイベント（Display/Erase/Highlight/Select/MoveTo など）が状態遷移を駆動し、Dirty を発火する。

### 4.6 AIファースト安全性

- 人間が Rust を書かない前提で、API設計と可視性で誤用を物理的に防ぐ。
- public API は最小限。便利だが危険な低レベル関数は公開しない。
- README/コメント依存ではなく型と構造で安全を担保する。

### 4.7 エージェント駆動テスト用組み込みHTTP API

- axum ベースのローカル HTTP+JSON サーバをアプリに同梱し、SceneContext の public API を薄くラップする。
- 想定エンドポイント例：`POST /api/entity`、`POST /api/select`、`GET /api/state`、`GET /api/screenshot`。
- ローカルホスト専用。認証は当面不要。将来 gRPC へ置換する余地は残す。
- ハンドラは SceneContext 経由の安全な操作のみ許可し、内部構造を直接公開しない。


---

## 5️⃣ Non-Functional Requirements

| 項目 | 要件 |
|------|------|
| FPS | 60fps 以上を実運用ターゲット。sync/render はフレーム境界で O(N_dirty) を保証する。|
| 拡張性 | ECSシステムを追加可能。systemは副作用を限定し、データ所有権を明示する。|
| 移植性 | Native / WebAssembly両対応可能な設計。wgpu feature set に依存する処理はゲートで分離。|
| 安定性 | debugビルドで状態不整合は panic で発見。整合性チェックを subsystem ごとに配置。|
| 並列性 | まずは同期フローで正しく動かす。並列化は計算系に限定し、wgpu queue/command encoder は単一点管理。|
| ロギング | tracing ベースで可観測性を確保。重要イベント（sync開始/完了、リソース再生成）を span で記録。|
| テスト | 単体：dirty伝搬と状態遷移のテーブルテスト。統合：mock wgpu backend で SceneContext API を検証。|

---

## 6️⃣ Out of Scope

- UIツールバー
- 完全なCADカーネル機能
- トレーシング/物理レンダリング

---

## 7️⃣ Success Definition

- SceneContext の public API だけで描画・選択可能
- Dirtyベースで必要最小限のGPU同期だけが走る
- ECSシステムが増えても構造破綻しない
- コマンドサーバ経由で定義済みシナリオ（例: 単一ボディ表示/選択、アセンブリ表示＋ハイライト）が再現でき、スクリーンショット比較で一致する

---

## 8️⃣ Documentation & Change Management

- API・状態遷移の更新は `docs/design/architecture-overview.md` と `docs/design/state-transitions.md` に即時反映。
- 破壊的変更は Reason/Impact/Migration を Markdown コメントで残すこと。
- 新システム追加時は責務、入力データ、dirty との関係を要件としてここに追記する。

---

## 9️⃣ Use Cases (Minimum Scenarios)
- 単一ボディ表示: Shape登録→表示→レンダ→スクリーンショット取得（HTTP経由）
- 単一ボディ選択: 表示済みボディに選択/解除→状態取得（HTTP）→スクリーンショット
- アセンブリ表示＋ハイライト: 複数Entityを登録・表示し、一部をハイライト→状態取得→スクリーンショット

## 🔟 Command Server Role / Boundaries
- 役割: テスト/自動化/AIエージェント操作用のフロント。SceneContextの public API を薄くラップ。
- 境界: 内部構造を露出しない。ローカルホスト専用、認証不要（現段階）。
- 将来: gRPCやUI統合も可能だが、HTTP+JSONの最小セットは維持。
