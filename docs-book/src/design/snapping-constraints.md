# Snapping & Constraints — Point Input Preprocessing

## 1️⃣ Scope

- Snapping と幾何拘束は **PointInput の前処理** として扱い、FSMの状態遷移には含めない。
- 目的は「ユーザーの意図に近い座標を生成すること」であり、SceneContext や CADコアのAPIはスナップ後の座標だけを見る。

## 2️⃣ Snap 種類と優先順位

- 対応するスナップ:
  - Endpoint
  - Intersection
  - Midpoint
  - Center
  - Perpendicular
  - Tangent
  - Nearest

- 優先順位（上にあるほど優先）:
  1. Endpoint
  2. Intersection
  3. Midpoint
  4. Center
  5. Perpendicular
  6. Tangent
  7. Nearest

- Snap設定で ON/OFF を切り替え可能にし、有効な種類の中から最も優先度の高い候補を採用する。

## 3️⃣ Snap 適用のタイミング

- `MouseMove` / `MouseDown` などで PointInput 候補が生成された段階で:
  1. 画面座標とCameraからワールド座標のレイ/位置候補を求める。
  2. 有効な Snap 種類ごとに候補点を収集する（近傍のエンティティを対象）。
  3. 優先順位に従ってベストなスナップ点を選ぶ。
  4. その点を `PointInputResult` として Command FSM に渡す。

- これにより FSM は「スナップ済み座標」だけを扱えばよく、Snapロジックは独立して発展させられる。

### PointInputResult 構造

- Snap処理の出力は常に以下の形に統一する:

```text
PointInputResult {
  raw: Point3,               // スナップ前の生座標
  snapped: Point3,           // 採用されたスナップ座標
  snap_type: Option<SnapKind>, // Endpoint / Midpoint / ... / None
}
```

- FSM/コマンドは通常 `snapped` を採用し、`snap_type` はプレビューやUIフィードバック用に利用する。

## 4️⃣ Ortho / Parallel / Perpendicular 拘束

- Ortho（直交）:
  - ON の場合、ドラッグ開始点からの移動ベクトルを Screen X/Y 方向に投影し、主方向（水平 or 垂直）にスナップする。
  - 線コマンド中の第2点などに適用。

- Parallel（平行）:
  - ガイドとなる既存線分と平行な方向に候補を制限する。

- Perpendicular（垂直）:
  - 既存線分に対して垂直方向となる点にスナップする。

適用順序の例:
- まず Ortho/Parallel/Perpendicular などで方向制約をかける。
- その後、方向制約に沿った点の中で Snap 種類（Endpoint/Midpoint/...）の候補を探す。

## 5️⃣ 設定とUI

- Snapping/Constraints は Model 内の設定として保持:
  - `snap_settings { endpoint: bool, midpoint: bool, ... }`
  - `constraint_settings { ortho: bool, parallel: bool, perpendicular: bool }`
- UI(egui) からトグルボタン等で変更し、Msgとして update に渡す。
