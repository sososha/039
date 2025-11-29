# HTTP API Specification — SceneContext Command Server

本書は axum ベースの HTTP+JSON API で公開する SceneContext のインターフェース仕様を定義する。

## 1️⃣ 共通事項

- ベースパス: `/api/scene`
- リクエスト/レスポンスはすべて JSON（スクリーンショットを除き）。
- エラー形式:

```jsonc
{
  "code": "UnknownEntity", // or ResourceMissing / InvalidState / Io / Backend
  "message": "human readable message",
  "id": 123                  // 任意: 対象IDなど
}
```

- HTTPステータスと `code` の対応（SceneError とのマッピング）:
  - `SceneError::UnknownEntity` → `404 Not Found`, `code="UnknownEntity"`
  - `SceneError::ResourceMissing` → `400 Bad Request`, `code="ResourceMissing"`
  - `SceneError::InvalidState` → `400 Bad Request`, `code="InvalidState"`
  - `SceneError::Io` → `500 Internal Server Error`, `code="Io"`
  - `SceneError::Backend` → `500 Internal Server Error`, `code="Backend"`

## 2️⃣ Endpoints

### 2.1 Submit / Update Entity

- `POST /api/scene/entity`

Req:

```jsonc
{
  "entity_id": null,           // null=新規作成, 数値=既存更新
  "shape": { /* Mesh or shape payload */ },
  "tess": { "max_angle": 0.05, "max_error": 0.001 }
}
```

Res (200 OK):

```jsonc
{ "entity_id": 123 }
```

エラー:
- `entity_id` が数値だが、存在しないIDの場合 → 404 + `code="UnknownEntity"`（暗黙の新規生成はしない）。
- tessellate が空メッシュ → 400 + `code="ResourceMissing"`。

### 2.2 Visibility / Highlight / Select

- `POST /api/scene/visibility`

Req:

```jsonc
{ "entity_id": 123, "visible": true }
```

Res: `{}` (200 OK)

- `POST /api/scene/highlight`
- `POST /api/scene/select`

Req:

```jsonc
{ "entity_id": 123, "value": true }
```

Res: `{}` (200 OK)

エラー:
- 対象EntityIdが存在しない → 404/UnknownEntity。
- Visible=false 状態で Highlight/Select を行った場合は SceneContext 側でエラーとし、400/InvalidState を返す。

### 2.3 Transform

- `POST /api/scene/transform`

Req:

```jsonc
{
  "entity_id": 123,
  "matrix": [
    [1.0, 0.0, 0.0, 0.0],
    [0.0, 1.0, 0.0, 0.0],
    [0.0, 0.0, 1.0, 0.0],
    [0.0, 0.0, 0.0, 1.0]
  ]
}
```

Res: `{}` (200 OK)

### 2.4 Remove

- `DELETE /api/scene/entity/{id}`

Res: `{}` (200 OK)

エラー:
- 存在しないID → 404/UnknownEntity。

### 2.5 State

- `GET /api/scene/state/{id}`

Res (200 OK):

```jsonc
{
  "visual": {
    "visible": true,
    "selected": false,
    "highlighted": false
  },
  "transform": [[...4x4...]],
  "has_mesh": true
}
```

エラー: 404/UnknownEntity。

### 2.6 Render & Screenshot

- `POST /api/scene/render`

Req:

```jsonc
{ "camera": { /* CameraParams 相当 */ } }
```

Res: `{}` (200 OK)

- `GET /api/scene/screenshot`

Res (200 OK):

```jsonc
{ "image_base64": "..." }
```

仕様:
- 画像フォーマット: PNG 固定。
- 解像度: 現在のレンダリングターゲットと同じ。
- コンテントタイプ: `application/json`（中身の `image_base64` がPNGをBase64エンコードしたもの）。

### 2.7 Pick

- `POST /api/scene/pick`

Req:

```jsonc
{ "screen_pos": [x, y] }
```

Res (200 OK):

```jsonc
{ "entity_id": 123 }
```

または:

```jsonc
{ "entity_id": null }
```

仕様:
- `screen_pos` はウィンドウピクセル座標系で、(0,0) が左上原点、(width, height) までの範囲を取る。
- `pick` は描画状態を変更せず、ハイライト更新はクライアントが `set_highlight` エンドポイントを明示的に呼ぶ。

