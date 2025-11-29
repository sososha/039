# State Transitions — SceneContext / VisualFlags / Dirty

## Entities & Flags
- EntityId: 安定ID（再生成なし）
- VisualFlags: Visible, Selected, Highlighted
- DirtyFlags: Geometry, Transform, Visual

## Event Definitions (high-level API)
- `SubmitShape(entity, shape)` : Shape登録 & tessellate（デフォルトで可視化）
- `Remove(entity)` : Entity/リソースの破棄
- `Display(entity)` / `Erase(entity)` : 表示ON/OFF
- `Highlight(entity, on/off)` : ハイライト切替（Visible 前提）
- `Select(entity, on/off)` : 選択切替（Visible 前提）
- `SetTransform(entity, transform)` : 変換更新（AISの MoveTo ではなく Transform 設定）
- `ClearSelectionAll()` : 全Entityの選択を解除

## Transition Table
| Event | Precondition | Postcondition (VisualFlags/World) | Dirty | Side Effects |
|-------|--------------|-------------------------------------|-------|--------------|
| SubmitShape | entity 未登録 | Entity作成、Mesh生成、Visible=on, Selected=off, Highlighted=off | Geometry+Visual | tessellate→mesh/normal生成、GPUバッファ割当予約、即表示 |
| Remove | entity 登録済 | Entity/リソースを破棄 | Geometry+Transform+Visual (full) | GPUリソース回収、ID解放 |
| Display(on) | Entity存在 | Visible=on | Visual | インスタンス可視化、instance/uniform更新 |
| Display(off)/Erase | Entity存在 | Visible=off | Visual | インスタンス非表示、必要なら描画リストから除外 |
| Highlight(on) | Entity存在かつ Visible=on | Highlighted=on | Visual | ハイライト用属性を更新 |
| Highlight(off) | Entity存在 | Highlighted=off | Visual | ハイライト属性をクリア |
| Select(on) | Entity存在かつ Visible=on | Selected=on | Visual | 選択用属性を更新 |
| Select(off) | Entity存在 | Selected=off | Visual | 選択属性をクリア |
| SetTransform(transform) | Entity存在 | Transform更新（行列差し替え） | Transform | 逆行列/法線行列再計算、instance buffer 再書き込み |
| ClearSelectionAll | なし | 全Entityの Selected=off (Highlightは維持) | Visual | 選択属性を一括クリア |

## Dirty Resolution Order
1. Geometry: tessellate→mesh更新→GPU buffer 再構築
2. Transform: モデル/法線行列を計算し instance buffer へ反映
3. Visual: Visible/Selected/Highlighted に応じて属性/uniform/instance を更新

render()/screenshot は内部で dirty を検査し、必要なら sync を実行してから描画する。sync 後も dirty が残っていれば debug_assert! で検知。外部からの手動 sync は不要。

## Error Handling
- 未知の EntityId へのイベント: debug=panic, release=Result::Err
- 必要リソース未生成での render: debug=panic
- Remove 後の再利用防止: EntityId 再利用を禁止し、回収済みIDの操作はエラー扱い

## HTTP API と状態遷移の対応
- HTTP経由の各イベントも SceneContext と同一の遷移テーブルに従う。
- `/api/render` は render 前に内部で sync を完了させ、dirty 非空なら debug_assert! を維持。
- `/api/screenshot` は最新フレームまたは直近の render 出力を返す。render が未実行なら内部で1フレーム描画する。

## State Machine (Display/Highlight/Select)

### States (per EntityId)
- S0: Invisible, Unselected, Unhighlighted
- S1: Visible, Unselected, Unhighlighted
- S2: Visible, Highlighted, Unselected
- S3: Visible, Selected (Highlight自由: ON/OFF)

### Events & Rules
- SubmitShape: S0 (implicitly) → S1 (Visible=on, Selected/Highlight=off) (Dirty=Geometry+Visual)
- Display(on):
  - S0 → S1 (Dirty=Visual)
- Display(off)/Erase:
  - S1/S2/S3 → S0 (Dirty=Visual, Clear Highlight/Select intentionally)
- Highlight(on):
  - Only when Visible (S1/S3) → S2 or keep S3 with Highlight=ON (Dirty=Visual). Not visible → Err (no auto-Display)
- Highlight(off):
  - S2 → S1, or S3 stays Selected (Dirty=Visual)
- Select(on):
  - Only when Visible (S1/S2) → S3 (Selected=ON, Dirty=Visual). Not visible → Err (no auto-Display)
- Select(off):
  - S3 → S1 or S2 depending on Highlight (Dirty=Visual)
- SetTransform (Transform change):
  - Any state → same visual state, Dirty=Transform
- ClearSelectionAll:
  - Any state → Selected cleared, Highlightそのまま (Dirty=Visual)

### Dirty Resolution Order
- Geometry → Transform → Visual（順序を崩さない）
- render/screenshot 前に dirty は空であるべき。非空なら sync して debug_assert! が発火。

### Illegal / Guarded Cases
- Visible=false で Highlight/Select された場合はエラー（no-op ではなく Result::Err / HTTPエラー）。暗黙の Display(on) はしない。
- 未知の EntityId へのイベントはエラー（debug=panic, release=Result::Err/HTTPエラー）
 
### Selection Mode (備考)
- 単一選択モードを採用する場合、`Select(on)` 実行時に他の Selected をすべて OFF にしてから ON にする。
- 複数選択モードなら、`Select(on)` は追加のみ。モードは上位(App/UI)が決定する。
