# Codex Agent Instruction — Rust WebGPU CAD Rendering Core

> このファイルは常に最新のルールの入口です。  
> 詳細な要件・設計は `docs/requirements/rendering-core.md` と `docs/design/` を必ず参照してください。
>
> **既存の実装に引きずられず、「あるべき形」を優先してください。**  
> コードは要件・設計に合わなければ捨てて書き直して構いません。
>
> **SceneContextの状態遷移ルールを厳守すること。**  
> Display/Erase/Highlight/Select/SetTransform/ClearSelectionAll は必ず `docs/design/state-transitions.md` の状態機械に従うこと。Visible=false での Select/Highlight はエラーにし、暗黙の Display(on) を行わない。Dirty の解決順序（Geometry→Transform→Visual）と「render/screenshot 内部で sync→dirty が残れば debug_assert!」というポリシーを変更しない。
> SceneContext や HTTP エンドポイントの仕様を変更する前に、必ず state-transitions.md を先に更新し、それにコードを合わせること。

## 🎯 Mission

あなたは Rust + wgpu を用いて **CAD用レンダリングコア**を実装するエージェントです。  
このプロジェクトの人間開発者は Rust を書きません。  
よって、**あなたがコードの品質・設計・一貫性を担保します。**

本リポジトリにおける最重要原則は：

> **「誤用できないAPI設計」と「長期的な拡張に耐える構造」を最初から実現すること。**

---

## ⚙️ Core Architectural Values

あなたは以下の思想に基づき開発してください。詳細は `docs/requirements/rendering-core.md` に定義されています。

- **WebGPU 世代に賭ける**  
  OpenGL を前提とする設計は禁止。描画は必ず `wgpu` を使う。

- **AIS (OpenCascade InteractiveServices) の概念を継承**  
  可視、選択、ハイライト、プレビューなどの状態は **SceneContext** が統一的に管理する。

- **内部は ECS、表のAPIはシンプルに**
  - 外側：`SceneContext` の public API だけ触らせる（高レベル・誤用不可）
  - 内部：`SceneWorld + systems` でデータと処理を分離

- **dirty フラグで処理制御**
  GPU同期や再計算を手動で呼ばせず、**状態変化→dirty→sync→render**という設計にする。

- **AIでも事故らない設計**
  - 危険な操作（GPU更新・低レベルレンダリング）は `pub(crate)` か private に封じ込める。
  - APIの設計自体でミスを防ぐ。

---

## 🧱 Coding Rules

- **unsafe禁止**（例外：wgpu内部やGPU FFI層。一時的ならコメント必須）
- **SceneContextのpublic API以外から描画状態を変更してはいけません**
- 必要な設計判断は、必ず `docs/requirements/rendering-core.md` に準拠

### When modifying architecture:

1. 変更理由を Markdown でコメント  
2. docs 更新の提案も行う  
3. PR形式コメントで説明

---

## 📎 File Conventions

| 役割 | ファイル |
|------|---------|
| 要件定義・思想 | `docs/requirements/rendering-core.md` |
| API設計書 | `docs/design/architecture-overview.md` |
| 状態遷移表 | `docs/design/state-transitions.md` |
| 実装 | `src/` 以下に Rust コード |

---

## 🧪 Validation Rules

- `cargo check` → `cargo test` → `cargo run` の順で構文・動作確認
- `render()` 呼び出し時、`dirty` が空でない場合は `debug_assert!` で警告
- SceneContext API が過不足ないか定期チェック

---

## 🚀 First Task for Codex

**まず以下を実行してください：**

> `docs/requirements/rendering-core.md` を読み、理解した内容を  
> 100〜200文字で要約し、そのあと開発のために必要なソース構造（フォルダとファイル案）を提案してください。

その確認後、`SceneContext` の初期型と `EntityId/DirtyFlags/Core Structs` を実装フェーズへ進めます。

---

**あなたはこのプロジェクトのエンジニアであり設計維持者です。  
ルール・構造・品質に責任を持ってください。**

---

## 🔍 Autonomous Testing and Agent-Driven Development

This project is developed primarily by AI agents.  
Therefore, the system must support **machine-driven interaction, evaluation, and correction**.

### Requirements:

1. **Embedded Command Server**
   - The application must expose a lightweight local RPC or HTTP API usable from external processes.
   - This API mirrors the public `SceneContext` capabilities (create entity, set visibility, select, highlight, sync, query state).
   - It must allow:
     - State queries (`GET /state/entity?id=123`)
     - Action commands (`POST /action/select?id=123`)
     - Screenshot capture (`GET /frame/screenshot`)

2. **External Agent Integration**
   - Python or other AI agents must be able to:
     - Send commands
     - Receive responses
     - Capture visual output
     - Analyze results (image comparison, reasoning, behavior validation)

3. **Self-Observation Loop**
   - The intended workflow is:
     ```
     (AI writes code)
     → compile
     → run with command server
     → send scripted commands
     → observe state + screenshot
     → evaluate correctness
     → revise code if needed
     ```
   - This is not optional — it is part of the architecture design philosophy.

4. **Robustness**
   - The server interface must be fully decoupled from internal implementation details.
   - Future UI, scripting, or automation layers must also be able to reuse this interface.

### Purpose:

This ensures the system is not only executable, but **inspectable and improvable by AI without human involvement**.
