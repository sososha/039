<h1 align="center">📐 Rust CAD Rendering Core (Design Phase)</h1>

<p align="center">
🚧 <strong>現在：設計フェーズ（実装前）</strong> 🚧<br>
Rust + wgpu + ECS による CAD 向け描画/操作基盤を設計中です。<br>
実装前に、設計書と仕様を確定させる段階です。
</p>

---

## 🧭 このプロジェクトについて

このプロジェクトは、WebGPU 世代に対応した CAD コアエンジンの設計と実装を目指しています。特徴は:

- **長期運用可能な描画基盤**
- **AI と人間が共同で開発できるアーキテクチャ**
- **壊れない API と安全な状態遷移モデル**
- **テスト容易性・仕様優先設計**

CAD 分野の歴史的な OpenGL 依存を脱却し、

> **WebGPU 世代の CAD Runtime と Rendering Architecture を再設計**

することを目的としています。

---

## 🏗 現在の進行状況

☑ 設計思想 / SceneContext API / HTTP API / 状態遷移(FSM) / 永続化モデル(JSON v0) / E2Eテスト仕様  
⬜ wgpu 実装  
⬜ CADコマンド実装  
⬜ UI 結合  
⬜ アプリとしての動作

現在は **実装前の最後の整備工程** です。

---

## 📂 仕様書

主要な設計・仕様は `docs/design/` にまとまっています:

| ファイル | 内容 |
|---------|------|
| `architecture-overview.md` | コア設計・責務・境界 |
| `http-api.md` | Axum HTTP ラッパ仕様 |
| `state-transitions.md` | SceneContext 状態遷移(FSM) |
| `command-state-machines.md` | CADコマンドFSM |
| `persistence-model.md` | JSON 保存形式 (v0) |
| `test-plan.md` | E2E シナリオ・検証方式 |
| `app-interactions.md` | Msg/FSM/SceneContext フロー |
| `design-rationale-architecture.md` | 設計意図と履歴 |

---

## 🤝 コントリビューションについて

- 今は **設計レビュー・仕様検証・小規模実験** を歓迎します。
- 本格実装は設計が安定した後に進めます。

---

## 🧊 設計ステータスとセーブポイント

- 現在の設計は「デザインフリーズ」状態です。
- セーブポイント: Git タグ `spec-v0` / ブランチ `design-spec-freeze`
- 設計変更は原則 `main` / `feature/*` で docs を更新し、新しい spec タグを切る運用です。

| ブランチ | 意味 | 状態 |
|---------|------|------|
| `design-spec-freeze` | 設計保存（破壊禁止） | 🔒 PRのみ |
| `main` | 正式実装 | 🏗 |
| `feature/*` | 試験・部分実装 | 🧪 |

---

## 🔍 テストと品質保証

実装は次の基準で検証されます:

- 🔹 ユニットテスト
- 🔹 HTTP APIテスト
- 🔹 シナリオベースの E2E 検証（決定論テスト）
- 🔹 将来: スクリーンショット比較（ピクセル一致 → SSIM へ拡張予定）

---

## 🌿 参加ルール（人間もAIも同じ）

- 仕様を読まずにコードを書かない
- 変更はまず仕様に反映し、その後実装
- 実装が仕様に合わない場合は **実装を修正する**

---

## ✨ 最後に

このプロジェクトは、**長期に使えるCADエンジンの設計** を目指しています。まず設計を固め、安心して実装できる未来を作ります。興味があれば、ぜひ仕様を覗いてください 🚀

