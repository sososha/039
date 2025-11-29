# Command State Machines — CAD Commands

参照: `docs/requirements/CAD_COMMAND_SPECIFICATION.md`

共通のトップレベル状態（グローバルFSM）:
- `Idle` : コマンド未実行
- `CommandActive(cmd, local_state)` : あるコマンドが実行中（内部でローカルFSMを持つ）

共通イベント:
- `CommandStart(cmd)` : `Idle → CommandActive(cmd, local_state=Initial)`
- `Finish` : コマンドを終了し `CommandActive → Idle`
- `Cancel` : どの local_state でもプレビューを破棄し `CommandActive → Idle`

ローカルFSM（各コマンド固有）で PointInput/OptionInput/Commit を処理し、連続作図なら `local_state=Initial` に戻す。グローバルFSMとローカルFSMを明確に分け、コマンド切替でローカル状態をリセットする。

---

## Drawing Commands

### Line (線分)
- 状態（ローカルFSM）: `WaitFirst` → `WaitSecond` → `Commit` → （Continuous? `WaitFirst` : `Finish`）
- イベント:
  - `CommandStart(Line)` : グローバル Idle → CommandActive(Line, local=WaitFirst)
  - `PointInput(p1)` : 始点確定、プレビュー線を **preview_id** で `SubmitShape(Visible=on)`
  - `MouseMove(p)` : `WaitSecond` のまま、プレビュー線を **再tessellate/頂点更新** で終点更新（シンプル優先）
  - `PointInput(p2)` : 終点確定、Commit
    - プレビューIDは Remove
    - 本番線を **final_id** で `SubmitShape(Visible=on)` し確定
  - `Continuous=true` : Commit 後に local_state=WaitFirst（同コマンド継続）
  - `Continuous=false` : `Finish` して Idle
  - `Cancel` : どの local_state でもプレビュー Remove → Idle
    
決定事項:
- プレビュー用の EntityId と確定用の EntityId を分ける。
- MouseMove での更新は Transform ではなく、頂点/ジオメトリを更新（線なので軽量）。
- 必須点数(2)が揃うまで Commit しない。
 - Backspace が押された場合、ローカルFSM内で「直前の点入力」を1ステップ戻す（Previewの形状のみ更新）。これはグローバルUndoとは別の概念とする。

### Polyline (連続線)
- 状態: `WaitFirst` → `WaitNext*` → (Close/Commit) → (`Continuous? WaitFirst : Finish`)
- イベント:
  - `PointInput(p1)` : 始点確定、プレビュー開始（preview_id）
  - `PointInput(pn)` : 頂点追加、プレビュー更新（再tessellate）
  - `Close` : 始点に閉じて Commit
  - `Enter/Commit` : 現在の頂点列で確定（preview Remove → final SubmitShape）
  - `Cancel` : プレビュー破棄、Idle

### Double Line (二重線)
- 状態: `WaitWidth` → `WaitFirst` → `WaitSecond` → Commit → (`Continuous? WaitFirst : Finish`)
- イベント:
  - `OptionInput(width)` : 二重線間隔を設定
  - `PointInput(p1/p2)` : 中心線の始点/終点を入力し、幅を左右にオフセットして2本生成（プレビュー→Commit）。

### Circle / Arc
- Circle（Center+Radius）: `WaitCenter` → `WaitRadiusPoint` → Commit
- Circle（3点）: `WaitP1` → `WaitP2` → `WaitP3` → Commit
- Circle（2点径）: `WaitP1` → `WaitP2` → Commit
- Arc（3点 / 中心+開始+終了 / 開始+終了+半径）: 入力パターンごとに状態を分岐

### Rectangle
- `WaitCorner1` → `WaitCorner2` → Commit（中心指定なら `WaitCenter` → `WaitCorner` → Commit）

### Polygon (正多角形)
- `WaitCenter` → `WaitRadiusPoint` → `WaitSideCount` → Commit
- OptionInput: 内接/外接を指定。サイド数が未入力でCommitした場合はエラー。

### Spline (スプライン)
- `WaitFirst` → `WaitNext*` → (Enter/Commit) → Finish
- PointInputごとに制御点/フィット点を追加しプレビュー更新（Spline再計算）。最低必要点数 (<3) のままEnterならエラー。Cancelでプレビュー破棄。

### Hatch
- `WaitBoundarySelect` | `WaitInternalPoint` → プレビュー（パターン適用） → Commit。境界が無効ならエラー。

### Text / Dimension
- Text: `WaitInsert` → `WaitHeight` → `WaitRotation` → `WaitContent` → Commit
- Dimension: `WaitP1` → `WaitP2` → `WaitTextPos` → Commit（線種によって角度/半径など追加入力）

---

## Modification Commands

### Move / Copy / Rotate / Scale / Mirror / Offset（共通パターン）
- 状態: Idle → SelectEntities → WaitBasePoint → WaitSecondPoint/Params → Preview → Commit
- イベント:
  - CommandStart(cmd)
  - SelectionDone(ids) : 対象確定（ない場合はErr）
  - BasePointInput(p0)
  - NextInput(p1 or params) : プレビュー更新（コピー数、角度、距離など）
  - Commit : エンティティ更新/生成
  - Cancel : プレビュー破棄、選択解除（必要に応じて）

IDポリシー:
- プレビュー用の EntityId と確定用の EntityId を分ける。Previewは Cancel/Commit で Remove。
- Move/Rotate/Scale など元エンティティを更新する場合も、プレビューは別IDで描き、本番 Commit で Document/SceneContext を更新する（元IDに Transform 反映か、再 Submit）。

### Array
- SelectEntities → ChooseType(Rect/Polar/Path) → Params入力 → Preview → Commit

### Trim / Extend
- SelectCuttingEdges → SelectTargets → Commit (切断/延長) 
  - 対象選択が空ならErr

### Fillet / Chamfer
- Params入力 (Radius / Dist1+Dist2) → SelectEdge1 → SelectEdge2 → Preview → Commit

### Stretch
- SelectWindow(Crossing) で頂点集合を決定（対象0ならErr）
- BasePointInput(p0) → DisplacementInput(p1) → Commit（頂点のみ移動）
- Cancelでプレビュー破棄/選択クリア

### Explode / Join
- SelectEntities → Commit（分解/結合実行）

### Delete
- SelectEntities → Commit(Remove) → Finish
- Selectionが空ならエラー。Undo/Redo で復元可能にする。

### Undo / Redo
- コマンド履歴FSMに依存。実行時はプレビュー/選択をクリアして `Idle` に戻す。
- CommandActive 中の Undo/Redo は原則禁止（まず Cancel/Finish して Idle に戻してから実行）。
 - Undo/Redo 実行後は Selection を必ず Clear し、Command FSM は Idle にあることを前提とする。

---

## Selection & Navigation FSM (UI層)

前提:
- Selection FSM は UI 層で独立して動き、**Command FSM は Selection結果(SelectionDone)** だけを入力として受ける。
- クリック/ドラッグ処理は Selection FSM が担当し、その結果として SceneContext に Select/Highlight を送る。コマンドローカルFSMの中で直接クリック処理をしない。

### Selection
- モード: `Idle` (選択なし) / `SelectingSingle` / `SelectingWindow` / `SelectingCrossing`
- イベント:
  - `Click(entity)` : 単一選択トグル（単一選択モードなら他をOFF）。SceneContext: Select(on/off)
  - `WindowDrag(box)` : 完了時に包含/交差を判定し、対象セットをSceneContextにSelect(on)適用
  - `Shift+Click(entity)` : 選択解除（Select(off)）
  - `Esc` : ClearSelectionAll → Idle

### Pan / Zoom (カメラ)
- Pan: `PanIdle` → `Panning` (MiddleDrag/Space+Drag) → `PanIdle`
  - カメラ位置更新を `view_state.camera` に反映し、CameraParams として SceneContext.render に渡す。CommandActive 中でも許可し、Undo/Redo 対象外とする。
- Zoom: ホイール/Window指定でカメラのproj/ビューを更新（FSMは簡易、主に入力→Camera更新）。こちらも CommandActive 中で許可し、Undo/Redo 対象外。

## Snapping & Constraints

- Snapsは「PointInput」を修正する前処理として動く。FSMに入れず、PointInputが確定するときに適用。
- Ortho/Parallel/Perpendicular などの拘束は、移動ベクトルや終点を拘束した上で PointInput に渡す。

---

## 共通エラー・ガード
- 必須入力が揃わずに Commit した場合はエラー。FSMが必須入力数を管理する。
- Selectionが必要なコマンドで対象ゼロならエラー。
- プレビュー用エンティティは、Cancel/Finish 時に必ず Remove する。
- 可視前提の Highlight/Select は、SceneContext 側のルール（Visibleでなければエラー）に従う。

---

## 最低限の状態遷移例（Line コマンド詳細）

```
Idle --CommandStart(Line)--> WaitFirst
WaitFirst --PointInput(p1)--> WaitSecond {Submit preview line Visible=on}
WaitSecond --MouseMove(p)--> WaitSecond {Update preview line endpoint}
WaitSecond --PointInput(p2)--> Commit {Finalize line, clear preview}
Commit --Continuous?--> (if true) WaitFirst else Idle
WaitSecond --Cancel/Esc--> Idle {Remove preview}
```

Polyline/Rectangleなども同様に「必須点数n」「Close/Enterで確定」という形でFSM化する。
