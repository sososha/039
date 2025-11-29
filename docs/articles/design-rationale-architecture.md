# Design Rationale — Architectural CAD Rendering Core

このドキュメントは、プロジェクト開始から現在までに行った主な設計判断と、その背景をまとめたものです。将来の振り返りや外部向け記事のベースとして利用します。

## 0️⃣ 開発のざっくりタイムライン

1. **レンダリング要件の言語化**  
   WebGPU世代前提・SceneContextを唯一のフロント・Dirty駆動レンダリングというコア要件を `rendering-core.md` に定義。

2. **CADコアのスコープ決定**  
   建築CAD専用（機械/設備はスコープ外）、f64幾何・BRep/CSG/Sketch・Document/Building/Level/Element構造を `cad-core.md` / `cad-architecture-overview.md` に整理。

3. **状態遷移とSceneContext FSM**  
   VisualFlags/DirtyFlagsを設計し、Display/Erase/Highlight/Select/SetTransform/ClearSelectionAll の状態機械を `state-transitions.md` に明示。AGENTS.mdで遵守を強制。

4. **HTTPコマンドサーバと外部制御**  
   axumベースのHTTP+JSON APIで SceneContext をラップし、AI/外部エージェントからの制御とスクリーンショット取得を可能にする方針を決定。

5. **コマンドFSM（二層構造）の設計**  
   グローバルFSM(Idle/CommandActive)と各コマンドFSM(Line/Polyline/Move/Trimなど)を `command-state-machines.md` に定義。Selection FSM はUI層に分離。

6. **App/UI レイヤと Msg/Update/SceneContext の流れ**  
   Elmアーキ(Model/Msg/update)＋immediate UI(egui)のハイブリッド構成を採用し、`app-ui.md` と `app-interactions.md` で Msg→FSM→SceneContext/CADコアのフローを設計。

7. **Picking・Snap・Pan/Zoom・Selectionの連携**  
   `pick(screen_pos)` APIをSceneContextに定義（v0 CPU → v1 GPUに移行）。Snapping/ConstraintsはPointInput前処理として分離し、Selection FSM/Command FSMとのルーティング方針を確立。

8. **Undo/Redo・Layer/Properties・永続化・テスト戦略**  
   CADコアでの差分ログベースUndo/Redo、Layer/ByLayerモデル、JSONベース永続化、代表E2Eシナリオを含むテスト計画を `cad-architecture-overview.md` / `layers-properties.md` / `persistence-model.md` / `test-plan.md` に落とし込んだ。

## 1️⃣ 全体のゴール

- Rust + wgpu を用いた **建築CAD専用レンダリングコア** を作る。
- 人間開発者はRustを書かない前提で、**AIが触っても壊しにくいAPI設計**を目指す。
- OpenGL世代の設計やREADME頼りの注意喚起をやめ、型/visibility/APIで誤用を防ぐ。

## 2️⃣ レイヤ構造の決定

- App/UI 層: Elmアーキテクチャ (Model / Msg / update / view)。
- CAD Core 層: 建築CADコア（Document/Building/Level/Grid/Element、f64 幾何、BRep/CSG/Sketch）。
- Rendering Core 層: SceneContext + wgpu による描画、VisualFlags/Dirty/HTTPコマンドサーバ。

### なぜこの三層か

- App/UIは頻繁に変わるため、CADコア/レンダリングコアから独立させたい。
- CADコアは建築BIM的な意味を持ち、Rendering CoreはピクセルとGPUに集中させたい。
- SceneContextを唯一の描画フロントにすることで、低レベルな誤用を防ぐ。

## 3️⃣ SceneContext と AIS 的設計

- OCCTのAIS思想を「機能コピーではなく思想として」採用:
  - Display/Erase/Highlight/Select など高レベルイベントで状態遷移させる。
  - 描画/選択状態は SceneContext が一元管理し、内部のGPU操作は隠蔽。
- VisualFlags(Visible/Selected/Highlighted) と DirtyFlags(Geometry/Transform/Visual) で状態を表現。
- 状態遷移表とFSMを `docs/design/state-transitions.md` に明記し、AGENTS.md で遵守を要求。

### 重要な決定

- SubmitShape はデフォルトで Visible=on（作ったら見える）。
- Display(off) で Select/Highlight も意図的にクリア（不可視＝選択も解除）。
- Visible=false での Highlight/Select はエラー扱い（no-op/暗黙Displayはしない）。
- render/screenshot は内部で dirty を見て sync→dirtyが残れば debug_assert!、外部からの手動syncは不要。

## 4️⃣ コマンドFSMと二層構造

- グローバルFSM: Idle / CommandActive(cmd, local_state)。Finish/CancelはイベントとしてIdlへ戻す。
- ローカルFSM: 各コマンド（Line/Polyline/Circle/Move/Trim…）の入力状態（WaitFirst/WaitSecond/Commit等）。
- Selection FSM と Command FSM を分離し、クリックルーティングは tool_state で切り替える。

### なぜFSMをここまで書くか

- 「1本線を引いたら次が引けない」といった典型的なバグがFSM不足から生まれるため。
- LineのFSMを教科書にし、他コマンドを同じ型で増やすことで、AIがコマンド追加するときの迷いを減らす。

## 5️⃣ App/UI と immediate vs 宣言的 UI

- 構造は Elm式（宣言的）: Model/Msg/update でアプリ全体を統治。
- UI描画には egui のような immediate UI を使う想定。ただし view 内では Msg を出すだけとし、ロジックはすべて update 側に置く。
- Pan/Zoom は view_state.camera の更新のみとし、CADモデルやUndo/Redoには影響させない。

## 6️⃣ Picking 戦略と CPU→GPU 移行

- SceneContext に高レベルAPI `pick(screen_pos) -> Option<EntityId>` を用意。
- v0: CPUベースのヒットテスト（CameraParams + ジオメトリ）。
- v1: GPUベースのIDバッファ（オフスクリーンに EntityId を色として描画し、ピクセルから復元）。
- App/UI は pick API だけを前提にし、内部実装がCPU/GPUどちらかは透過的とする。

理由:
- 最初からGPUピックに振ると設計負荷が大きいが、最終的にはGPUピックがないと大量要素に対応できない。
- APIを先に固定し、v0でCPU実装→レンダリングコアが安定した段階でGPU実装に差し替えられるようにするため。

## 7️⃣ EntityId と ElementId/TopoId の分離

- Rendering Coreは EntityId をキーに描画状態のみ管理。
- CAD Core は ElementId / FaceId / EdgeId など意味のあるIDを持ち、EntityId→ElementId(+TopoId)のマッピングを管理。
- Selection/Commandは:
  - pick → EntityId
  - CADコアのマップ → ElementId/TopoId
  - CAD状態更新 & SceneContextへのSelect/Highlight

## 8️⃣ Undo/Redo とコマンド中の扱い

- Undo/Redoはコマンド履歴FSMに依存し、実行時はSelection/PreviewをクリアしてIdleへ戻す。
- CommandActive中のUndo/Redoは原則禁止とし、まずCancel/FinishでIdleに戻してから実行する。

理由:
- コマンド途中の中途半端な状態をUndo/Redo対象にすると、状態爆発とバグの温床になるため。

さらに:
- CADコアでは Element/Component の変更を差分ログとして扱い、1コマンド=複数Diffの束として履歴に積む。
- Undo はDiffを逆順に適用し、Redoは順方向に適用する。Rendering CoreはCAD状態から再syncされるだけで、Undo/Redoそのものは知らない。
- コマンド内部でのBackspace/Escなど「1ステップ戻る/キャンセル」はローカルFSMの責務とし、グローバルUndoとは分離する。

## 9️⃣ Layer/プロパティ・スナップ/拘束・永続化

- Layer/プロパティ:
  - Layer: 作成/削除/表示/非表示/ロック。Elementとの紐付けはCADコア側の責務。v0では locked layer 上の要素は選択/ハイライト/編集すべて不可とする（将来拡張余地あり）。
  - Visibleの最終結果は Layer.Visible と Element単位の Visible を組み合わせてSceneContextへ渡す方針（AND論理）。
- スナップ/拘束:
  - SnappingはPointInput前処理として扱い、FSMに入れない。出力は `PointInputResult { raw, snapped, snap_type }` で統一し、FSM側は snapped のみを使う。
  - Ortho/Parallel/Perpendicularなどは移動ベクトル/終点を拘束してからPointInputに渡す。
- 永続化:
  - 初期はJSONベースで Project/Document/Building/Level/Layer/Element を保存する想定。Elementはフラット配列＋ID参照構造にし、深いネストを避ける。永続IDは UUID 等で管理し、SceneContext 内部のGPUインデックス等とは切り離す。
  - 安定後にSTEP等の外部フォーマットへの橋渡しを検討。

## 🔟 今後の設計の方向性

- 既に固まっている:
  - Rendering Coreの責務とSceneContext API
  - 建築CAD CoreのスコープとID設計
  - コマンドFSMとSelection FSMの枠組み
  - App/UIのElmアーキ＋immediate UIの組み合わせ

- これから詰める:
  - Undo/Redoの具体的な履歴モデルと永続化戦略
  - Layer/プロパティの詳細仕様とSceneContextとの連携
  - スナップ/拘束の優先順位とUI操作
  - テストシナリオ（線作図→編集→Undo/Redo→再描画）のテーブル化

このドキュメントは、これまでの対話で出てきた「なぜそうしたか」を凝縮したメモです。詳細な仕様や状態遷移は各requirements/designドキュメントを参照しつつ、ここを「設計の物語」として保持しておきます。

---

## 1️⃣1️⃣ Design History Overview（文章バージョン）

ここまでの設計の流れを、ストーリーとしてまとめる。

1. **理想のCAD像と言語化**  
   最初に「AIがメイン実装者で、人間はRustを書かない」という前提から、OpenGL設計やREADME頼りを捨て、WebGPU + SceneContext 一本化 + Dirty駆動 + AIS思想という大枠を `rendering-core.md` に落とした。

2. **建築CADコアのスコープ確定**  
   機械/設備はスコープ外とし、建築に特化した Document/Building/Level/Element/Layer/Relation を `cad-core.md` と `cad-architecture-overview.md` に定義。f64幾何/BRep/CSG/Sketchはコアの責務、レンダコアは Mesh/Polyline への投影だけを見る。

3. **SceneContext状態遷移の固定**  
   VisualFlags/DirtyFlags を軸に、Display/Erase/Highlight/Select/SetTransform/ClearSelectionAll の状態遷移を `state-transitions.md` に書き下ろし、Visible前提やDirty順序、Illegalケースを仕様化。AGENTS.md で「必ずこれに従う」ことをルール化した。

4. **HTTPコマンドサーバとAI駆動テストの導入**  
   SceneContext の public API をほぼそのまま HTTP+JSON にラップする axum コマンドサーバを設計。AI/Pythonエージェントが作図/状態取得/スクリーンショットを行い、自動で挙動評価・修正できる「自己観察ループ」を要件に組み込んだ。

5. **コマンドFSMとSelection FSMの二層化**  
   Line/Polyline/Move/Trim 等のコマンドを、グローバルFSM(Idle/CommandActive)＋ローカルFSM(WaitFirst/WaitSecond/Commit/…)で表現し、Selection FSM はUI層に分離。クリックは tool_state に応じて SelectionかCommandにルーティングする方針を `command-state-machines.md` にまとめた。

6. **App/UI: Elmアーキ＋egui のハイブリッド**  
   アプリ全体は Model/Msg/update/view の Elmアーキで統治し、UI描画には immediate UI (egui) を使うが、view では Msg を発行するだけとし、SceneContext/CADコアへの操作は update 内に閉じ込める設計を `app-ui.md` と `app-interactions.md` で定義。Pan/Zoom は view_state.camera だけを変え、Undo/Redoとは独立した操作にした。

7. **Picking/Snap/Constraints/Selection の切り分け**  
   SceneContext に `pick(screen_pos) -> Option<EntityId>` の高レベルAPIを定義し、v0 CPU→v1 GPU(IDバッファ)へ移行可能に。Snapping/Constraints を PointInput前処理として分離し、FSM側はスナップ済みの `PointInputResult { raw, snapped, snap_type }` だけ見るようにした。Selection FSM は UI 層で独立し、Command FSM は Selection結果のみを受け取る構造にした。

8. **Undo/Redo・Layer/プロパティ・永続化の決定**  
   Undo/RedoはCADコアの Element/Component 差分ログで行い、Rendering Coreは結果に追従するだけと決めた。Layer/ByLayerモデルは AutoCAD互換の AND論理（Layer.visible × Element.visible）で設計し、locked layer は v0 では選択/ハイライト/編集すべて不可とした。永続化はJSONベースのフラット＋ID参照構造にし、Project/Document/Building/Level/Layer/Element を安定IDで保存する方針を決定した。

9. **SceneContext API / HTTP API / テスト仕様の固定**  
   SceneContext の外部契約（submit_shape / set_visibility / set_highlight / set_selected / set_transform / remove / render / pick / get_state）を `architecture-overview.md` に固定し、それを薄くラップする HTTP API を `http-api.md` に定義。E2Eシナリオテストは YAML/JSON ベースで「線→Move→Trim→Undo/Redo→再描画」の代表シナリオを formal spec に落とし、state_assert/screenshot_assert の意味も `test-plan.md` に定義した。

10. **設計仕様フリーズのスナップショット**  
    この時点の設計状態を Git タグ `spec-v0` およびブランチ `design-spec-freeze` としてGitHub上に保存した。今後大きな実装・試行錯誤を行っても、このタグ/ブランチに戻ることで「設計仕様完了時点」から再スタートできるようにしている。設計変更は原則 `main` や `feature/*` ブランチで docs を更新し、新たな spec タグを切る運用を想定している。

この一連の流れにより、「理想像」から始まった設計が、SceneContext/CADコア/App/UI/HTTP/永続化/テストという各層の**具体的な契約とデータ構造**まで降りてきた。今後は、この仕様群を基に実装・検証フェーズへ進みつつ、必要な箇所を小さくアップデートしていく。
