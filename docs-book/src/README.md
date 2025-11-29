# Rust CAD Rendering Core — Design Specs

この mdBook は、Rust + wgpu + ECS ベースの
**CAD レンダリング・操作コア** に関する設計仕様をまとめたものです。

> 実装よりも先に、「壊れない設計」と「AIが守れるルール」を固めるフェーズです。

---

## 章構成

本書は次のような構成になっています（詳細は左のナビゲーションを参照してください）。

- 設計概要・責務分離（Architecture Overview）
- CADコア設計（CAD Architecture Overview）
- SceneContext の状態遷移 / コマンドFSM / Appメッセージフロー
- HTTP API 仕様
- レイヤ / スナップ / 永続化(JSON) モデル
- テスト計画 / E2E シナリオ
- Rendering/CAD/App/UI の要件定義
- 設計の背景・判断理由

---

## 仕様の入り口

設計の読み始めとしては、次の順序を推奨します。

1. [Architecture Overview](design/architecture-overview.md)  
2. [CAD Architecture Overview](design/cad-architecture-overview.md)  
3. [Command State Machines](design/command-state-machines.md) と [State Transitions](design/state-transitions.md)  
4. [HTTP API Spec](design/http-api.md)  
5. [Persistence Model](design/persistence-model.md) / [Test Plan](design/test-plan.md)  

背景や意思決定の経緯を知りたい場合は:

- [Design Rationale](articles/design-rationale-architecture.md)  
- [Why This Architecture](articles/why-this-architecture.md)  

から読むのがおすすめです。

---

## メモ

元のリポジトリ構成との対応関係は以下の通りです。

- `docs/design/*` → `design/*`  
- `docs/requirements/*` → `requirements/*`  
- `docs/articles/*` → `articles/*`  

この mdBook はあくまで「閲覧用ビュー」であり、元の Markdown ファイルはそのまま残しています。

