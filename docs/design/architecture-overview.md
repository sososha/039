# Architecture Overview — Rust WebGPU CAD Rendering Core

## Purpose
- 実装フェーズでの共通参照点。SceneContext を唯一のフロントとし、内部は ECS (SceneWorld + systems) で分離する設計の全体像を示す。
- Elm型統治（Model/Msg/update）と AIS 思想（コンテキスト経由の高レベルイベント駆動）を両立し、誤用不能な API を提供する。

## Layering / Responsibilities
- App (Model/Msg/update): アプリ状態の管理。update 内から SceneContext の高レベルAPIを呼ぶ。
- SceneContext (public): 描画・可視/選択/ハイライト操作のフロント。低レベルリソースは非公開。
- SceneWorld (internal ECS storage): Entity、Mesh、Material、VisualFlags、Transforms、DirtyFlags を保持。
- Systems (internal):
  - sync_gpu: Dirty を解決し、Geometry→Transform→Visual の順で GPU へ反映。
  - render: wgpu パイプラインを用いた描画。render 前に dirty 非空なら debug_assert!
  - selection/highlight: VisualFlags 更新と Dirty 発火。
- Command Server (public surface, thin): axum ベースのローカル HTTP+JSON API。SceneContext を薄くラップし、操作/状態取得/スクリーンショット取得を提供。内部構造は露出しない。
- GPU Layer (internal, pub(crate)): device/queue/pipeline/allocator を一元管理。

```mermaid
flowchart LR
    subgraph App["App / UI (Elm: Model/Msg/update)"]
        U[Update<br/>Command FSM<br/>Selection FSM]
    end

    subgraph CAD["CAD Core\n(Document / Building / Level / Element)"]
    end

    subgraph SC["SceneContext (public API)"]
        SW[SceneWorld\n(ECS storage)]
        SYS[Systems\n(sync_gpu / render / selection)]
    end

    subgraph HTTP["Command Server\n(axum HTTP+JSON)"]
    end

    subgraph GPU["GPU Layer\n(wgpu device/queue/pipeline)"]
    end

    U -->|CAD操作| CAD
    U -->|描画/選択API| SC
    CAD -->|KernelShape / Tess| SC
    SC <--> HTTP
    SYS -->|command buffers| GPU
    SW <-->|Entity/Visual/Dirty| SYS
```

## Data & IDs
- EntityId: 安定ID（再生成なし）、コピー可能な値型のみ露出。
- VisualFlags: Visible / Selected / Highlighted をビット管理。
- DirtyFlags: Geometry / Transform / Visual に分類。sync で Geometry→Transform→Visual の順に処理。
- Resources: Mesh/Normals/Material は immutable。差し替え時のみ新リソース生成。

## API Surface (SceneContext)

### Core methods
- Shape登録: `submit_shape(id: Option<EntityId>, shape: &impl KernelShape, tess: &TessParams) -> Result<EntityId, SceneError>`
  - `id=None` の場合は新規 EntityId を割り当てる（新規作成）。\n+  - `id=Some(existing_id)` の場合は既存エンティティのジオメトリ/リソースを差し替える（更新）。\n+  - `id=Some(id)` だが `id` が存在しない場合は **新規作成せず** `SceneError::UnknownEntity` を返す（暗黙の生成は禁止）。\n+  - tessellate が空メッシュを返した場合は `SceneError::ResourceMissing`。
- 表示/非表示: `set_visibility(id: EntityId, visible: bool) -> Result<(), SceneError>` → Visual dirty。
- ハイライト/選択:
  - `set_highlight(id: EntityId, highlighted: bool) -> Result<(), SceneError>` → Visual dirty。
  - `set_selected(id: EntityId, selected: bool) -> Result<(), SceneError>` → Visual dirty。
- 変換: `set_transform(id: EntityId, transform: Transform) -> Result<(), SceneError>` → Transform dirty。
- 破棄: `remove(id: EntityId) -> Result<(), SceneError>` → GPU/World から安全に回収。

### Frame & picking
- 描画: `render(&mut self, camera: &CameraParams) -> Result<(), SceneError>`
  - `render` 内部で dirty を検査し、必要なら `sync_gpu()` を実行する。
  - `sync_gpu()` 後に dirty が残っていれば debug_assert! で検知。
- ピック: `pick(&mut self, screen_pos: glam::Vec2) -> Result<Option<EntityId>, SceneError>`
  - v0: CPUベースのヒットテスト（CameraParams + ジオメトリで最近傍を決定）。
  - v1: GPUベースのIDバッファピックに置き換え可能。
  - Cameraは内部の最新状態を用いる（App側は座標だけ渡す）。
  - `pick` 自体は描画状態（Highlight/Select 等）を変更しない純粋な問い合わせとし、ハイライト更新は App 側が明示的に `set_highlight` を呼ぶ。

### Introspection
- 状態取得: `get_state(id: EntityId) -> Result<EntityState, SceneError>`
  - `EntityState { visual: VisualFlags, transform: Transform, has_mesh: bool }` を返す。

### 外部制御
- Command Server 経由で SceneContext API を HTTP+JSON でラップし、AI/外部プロセスから操作・状態取得・スクリーンショット取得を行う（ローカル専用、認証なし想定）。

### Picking API
- 上記 `pick(screen_pos)` を提供。
- App/UI 層は API だけに依存し、実装がCPU/GPUどちらかは透過的とする。

### API 署名の方針
- Result 型でエラーを返却。`SceneError` に UnknownEntity / ResourceMissing / InvalidState / Io / Backend を含める。
- borrow 方針: SceneContext は `&mut self` 受けを基本とし、同時操作を抑止。HTTPハンドラは単一スレッドかミューテックス越しに扱う。
- Camera: `CameraParams { view: Mat4, proj: Mat4, viewport: UVec2 }` の単純構造体を受け付ける。
- Transform: 行列渡しを基本とし、必要なら `Transform::from_translation_rotation_scale` のヘルパを提供。
- KernelShape: `fn tessellate(&self, params: &TessParams) -> MeshData` を要求。`TessParams { max_angle, max_error }` を最低限とする。

### Data Model Sketch
- EntityId: `u64` 安定ID（再利用禁止）。外部へはコピー可能値型のみ。
- VisualFlags: `{ visible: bool, selected: bool, highlighted: bool }` をビット管理。
- DirtyFlags: `Geometry | Transform | Visual`（bitflags）。
- Transform: 4x4 行列。scene座標系は右手系、単位はメートル想定。
- CameraParams: view/proj 行列、viewport (UVec2)。投影種別は行列に含める。
- MeshData: `{ vertices: Vec<Vertex>, indices: Vec<u32>, normals?: Vec<Vec3> }`。immutable、差し替え時に新リソース。
- TessParams: `{ max_angle: f32, max_error: f32 }` を最低限。

### HTTP Endpoint Schemas (axum, HTTP+JSON)
- `POST /api/entity`
  - Req: `{ shape: ShapePayload, tess_params?: TessParams, entity_id?: u64 }`
  - Res: `{ entity_id: u64 }`
- `DELETE /api/entity/{id}` → `{}`
- `POST /api/select` `{ entity_id: u64, selected: bool }`
- `POST /api/highlight` `{ entity_id: u64, highlighted: bool }`
- `POST /api/visibility` `{ entity_id: u64, visible: bool }`
- `POST /api/transform` `{ entity_id: u64, matrix4x4: [[f32;4];4] }`
- `POST /api/render` `{ camera: CameraParams }` → `{ frame_id: u64 }`
- `GET /api/state/{id}` → `{ visual_flags: {visible, selected, highlighted}, transform: [[f32;4];4], has_mesh: bool }`
- `GET /api/screenshot` → `image/png` (binary) もしくは `{ image_base64: string }`
- Error: HTTPステータス + `{ code: string, message: string }`。`code` は SceneError に対応。

### Command Server (HTTP+JSON, axum)
- 基本パス: `/api`。ローカルホスト専用、認証なし。
- Endpoints (例):
  - `POST /api/entity` {shape, tess_params?, entity_id?} → {entity_id}
  - `POST /api/select` {entity_id, selected: bool}
  - `POST /api/highlight` {entity_id, highlighted: bool}
  - `POST /api/visibility` {entity_id, visible: bool}
  - `POST /api/transform` {entity_id, matrix4x4}
  - `DELETE /api/entity/{id}` → {}
  - `POST /api/render` {camera} → {frame_id}
  - `GET /api/state/{id}` → {visual_flags, transform, has_mesh}
  - `GET /api/screenshot` → PNG (binary) or base64 PNG in JSON (`{image_base64}`)
- エラー: HTTPステータス + `{code, message}`。code は SceneError に対応。
- 実装上の制約: ハンドラは SceneContext の safe API のみを呼び、内部リソースへの直接アクセスを禁止。

## Error & Safety Policy
- debug: 不整合（未知の EntityId、未生成リソース、dirty未解決での render）は panic で発見。
- release: Result で上位へエラー通知。危険操作は pub(crate)/private に封じる。
- unsafe は禁止（wgpu/FFI 例外は理由コメント必須）。

## Concurrency / Async
- 初期フェーズは同期フローを最優先。並列化は計算系のみ、queue/encoder は単一点管理。
- 将来 async/並列を視野に入れるが、APIを複雑化しない。

## Observability / Logging
- tracing ベース。sync開始/完了、リソース再生成、render呼び出しを span で記録。

## Testing Strategy
- 単体: Dirty伝搬、VisualFlags/EntityId の状態遷移をテーブルテスト。
- 統合: mock wgpu backend で SceneContext API を経由した end-to-end 動作を検証。

## Open Points
- Camera API 仕様（パラメータ/投影種別）の固定。
- Selection/Hit-test 実装戦略（CPU vs GPU pick）。
- Tessellation パラメータの外部設定API。

<script src="https://unpkg.com/mermaid@9/dist/mermaid.min.js"></script>
<script>
  if (window.mermaid) {
    mermaid.initialize({ startOnLoad: true });
  }
</script>
