# Layers & Properties — Architectural CAD Core

## 1️⃣ Layer Model

- Layer は要素(Element)を論理的/表示的にグルーピングする単位。

### Layer 構造
- `Layer { id: LayerId, name: String, visible: bool, locked: bool, color: ColorSpec, linetype: LineTypeSpec, lineweight: LineWeightSpec }`
- Element は `element.layer_id: LayerId` を持つ。

### フラグの意味
- `visible`:
  - false の場合、その Layer に属する Element は **必ず画面上で非表示**。
  - true の場合のみ Element の Visible フラグが有効になる。
- `locked`:
  - true の場合、その Layer に属する Element は編集対象から除外（Move/Trim 等のコマンドは対象に含めない）。
  - v0 仕様では選択およびハイライトも不可（クリック/窓選択のヒットテスト対象から除外）とする。
  - 将来、"lockedだが選択だけ許可" などの拡張が必要になった場合は、追加フラグで制御する。

## 2️⃣ Element側の可視フラグとの優先順位

- Element は個別に `visible: bool` を持つ（レイヤ可視と独立）。
- SceneContext に渡す最終的な Visible 判定は:

```text
final_visible(element) = layer.visible && element.visible
```

- したがって:
  - Layer.visible=false なら Element.visible が true でも描画されない。
  - Layer.visible=true のときにのみ Element.visible による個別オン/オフが効く。

## 3️⃣ SceneContext との連携

- CADコアは Layer/Element の可視状態から EntityId ごとの final_visible を計算し、SceneContext に `set_visibility(EntityId, bool)` を送る。
- SceneContext は Layer/Element の概念を持たず、与えられた Visible をそのまま描画状態として扱うだけとする。

## 4️⃣ プロパティ: 色/線種/線幅/ByLayer

- 各 Element は以下のプロパティを持つ:
  - `color: ColorSpec` (ByLayer or Explicit)
  - `linetype: LineTypeSpec` (ByLayer or Explicit)
  - `lineweight: LineWeightSpec` (ByLayer or Explicit)

### 解決ルール

- 最終的な描画プロパティは Layer と Element から解決:

```text
resolved_color(element) =
  match element.color {
    ByLayer => layer.color,
    Explicit(c) => c,
  }
```

（linetype / lineweight も同様）

- 解決後のプロパティを SceneContext へ渡し、Vertex/Instance属性やUniformとして反映する。

## 5️⃣ Layer 操作コマンド

- CreateLayer(name) → 新規 Layer を追加。
- DeleteLayer(id) → Layer 削除（要素の扱い: 移動先レイヤ or エラーとするポリシーが必要）。
- RenameLayer(id, name)
- SetLayerVisible(id, bool)
- SetLayerLocked(id, bool)
- SetLayerProperties(id, color/linetype/lineweight)

これらは CAD Core のAPIとして定義し、Undo/Redoの差分ログ対象とする。
