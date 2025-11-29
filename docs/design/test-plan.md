# Test Plan & Failure Points (Rendering Core / CAD Core / HTTP)

## High-Risk Areas & Mitigations
- IDマッピング: EntityId と ElementId が混線しないか。→ 単体テストで UnknownEntity エラー、再利用禁止を確認。
- Dirty処理: dirty未解決で render が走る経路。→ render前に debug_assert! あり。syncが dirty をクリアすることをテスト。
- シリアライズ: glam行列/bitflagsのserde。→ HTTP経由の往復テストを用意。
- 形状の空メッシュ: tessellate が空を返したときのエラー。→ submit_shape のエラーをテスト。
- HTTPラッパ: axum経由で SceneContext を正しく呼べるか。→ Router oneshot テストで entity作成→state参照を確認。

## Minimum Test Scenarios
- SceneContextユニット
  - 新規 submit_shape が id を返し、dirty=GEOMETRY|TRANSFORM|VISUAL が立つ。
  - set_visibility/highlight/select/transform で dirty=VISUAL/TRANSFORM が立つ。
  - remove 不在なら UnknownEntity。
  - tessellate が空メッシュなら ResourceMissing。
- HTTP 経由
  - POST /api/entity で mesh 登録→GET /api/state/{id} で visual=false/has_mesh=true を取得。
  - 無効IDへの操作で 404/400 が返る。
- 今後 (未実装/検討)
  - render/screenshot が実際にPNGを返す。
  - EntityId→ElementId マッピングの往復 (CADコア連携後)。
  - f64→f32 TessParams/誤差の上限チェック。

## CAD Core / Command FSM / End-to-End Scenarios

- CAD Core 単体
  - Element追加/削除/更新の差分ログが正しく生成されること。
  - Undo/Redo が差分を逆順/順方向に適用し、Element集合が元に戻ること。
  - Layer.visible/locked が final_visible や編集可否に反映されること。

- Command FSM
  - Line: WaitFirst→WaitSecond→Commit の基本シーケンスと Cancel/Backspace の挙動。
  - Polyline: 点追加/Close/Enter/Cancel で頂点リストとプレビューが期待通りに変化すること。
  - Move/Copy/Trim: Selection→BasePoint→NextInput→Commit のフローと、Selectionが空のときのエラー。

- UI経由 HTTP シナリオ
  - シナリオ: 線作図→Move→Trim→Undo/Redo→再描画
    - `/api/entity` で線を2本登録。
    - `/api/transform` 相当のコマンドでMove操作をエミュレート。
    - Trimに相当するCADコア操作のHTTPラッパを経由して形状を変更。
    - Undo/Redo API (将来追加) 経由で状態が戻る/進むことを検証。
    - 各ステップで `/api/scene/state/{id}` と `/api/scene/screenshot` を参照し、一貫性が保たれていることを確認。
    - CameraParams やクリック座標はテストごとに固定値を使用し、結果が決定論的になるようにする（揺れる座標系は許容しない）。

### state_assert の意味
- `expect_json` は「指定されたフィールドについての部分一致」とする（レスポンスがそれ以上のフィールドを含んでもよい）。
- 数値については v0 では厳密比較（==）とし、誤差問題が出てきた場合にトレランス付き比較へ拡張する。

### screenshot_assert の意味
- v0 では、期待画像（fixtures）との比較は以下とする:
  - decodeしたPNGの解像度・フォーマットが一致すること。
  - バイト列（ピクセルデータ）が完全一致であること。
- 将来的には、画質指標(SSIM等)による近似比較や差分ハイライトなどに拡張する余地を残す。
